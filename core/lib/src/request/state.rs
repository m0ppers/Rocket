use std::ops::Deref;

use request::{self, FromRequest, Request};
use outcome::Outcome;
use http::Status;

/// Request guard to retrieve managed state.
///
/// This type can be used as a request guard to retrieve the state Rocket is
/// managing for some type `T`. This allows for the sharing of state across any
/// number of handlers. A value for the given type must previously have been
/// registered to be managed by Rocket via the
/// [manage](/rocket/struct.Rocket.html#method.manage) method. The type being
/// managed must be thread safe and sendable across thread boundaries. In other
/// words, it must implement `Send + Sync + 'static`.
///
/// # Example
///
/// Imagine you have some configuration struct of the type `MyConfig` that you'd
/// like to initialize at start-up and later access it in several handlers. The
/// following example does just this:
///
/// ```rust
/// # #![feature(plugin, decl_macro)]
/// # #![plugin(rocket_codegen)]
/// # extern crate rocket;
/// use rocket::State;
///
/// // In a real application, this would likely be more complex.
/// struct MyConfig {
///     user_val: String
/// }
///
/// #[get("/")]
/// fn index(state: State<MyConfig>) -> String {
///     format!("The config value is: {}", state.user_val)
/// }
///
/// #[get("/raw")]
/// fn raw_config_value<'r>(state: State<'r, MyConfig>) -> &'r str {
///     // use `inner()` to get a lifetime longer than `deref` gives us
///     state.inner().user_val.as_str()
/// }
///
/// fn main() {
///     let config = MyConfig {
///         user_val: "user input".to_string()
///     };
///
/// # if false { // We don't actually want to launch the server in an example.
///     rocket::ignite()
///         .mount("/", routes![index, raw_config_value])
///         .manage(config)
///         .launch();
/// # }
/// }
/// ```
///
/// # Within Request Guards
///
/// Because `State` is itself a request guard, managed state can be retrieved
/// from another request guard's implementation. In the following code example,
/// `Item` retrieves the `MyConfig` managed state in its `FromRequest`
/// implementation using the [`Request::guard()`] method.
///
/// [`Request::guard()`]: /rocket/struct.Request.html#method.guard
///
/// ```rust
/// use rocket::State;
/// use rocket::request::{self, Request, FromRequest};
///
/// # struct MyConfig{ user_val: String };
/// struct Item(String);
///
/// impl<'a, 'r> FromRequest<'a, 'r> for Item {
///     type Error = ();
///
///     fn from_request(request: &'a Request<'r>) -> request::Outcome<Item, ()> {
///         request.guard::<State<MyConfig>>()
///             .map(|my_config| Item(my_config.user_val.clone()))
///     }
/// }
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
    /// Retrieve a borrow to the underyling value with a lifetime of `'r`.
    ///
    /// Using this method is typically unnecessary as `State` implements `Deref`
    /// with a `Target` of `T`. This means Rocket will automatically coerce a
    /// `State<T>` to an `&T` as required. This method should only be used when
    /// a longer lifetime is required.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::State;
    ///
    /// struct MyConfig {
    ///     user_val: String
    /// }
    ///
    /// // Use `inner()` to get a lifetime of `'r`
    /// fn handler1<'r>(config: State<'r, MyConfig>) -> &'r str {
    ///     &config.inner().user_val
    /// }
    ///
    /// // Use the `Deref` implementation which coerces implicitly
    /// fn handler2(config: State<MyConfig>) -> String {
    ///     config.user_val.clone()
    /// }
    /// ```
    #[inline(always)]
    pub fn inner(&self) -> &'r T {
        self.0
    }
}

// TODO: Doc.
impl<'a, 'r, T: Send + Sync + 'static> FromRequest<'a, 'r> for State<'r, T> {
    type Error = ();

    #[inline(always)]
    fn from_request(req: &'a Request<'r>) -> request::Outcome<State<'r, T>, ()> {
        match req.get_state::<T>() {
            Some(state) => Outcome::Success(State(state)),
            None => {
                error_!("Attempted to retrieve unmanaged state!");
                Outcome::Failure((Status::InternalServerError, ()))
            }
        }
    }
}

impl<'r, T: Send + Sync + 'static> Deref for State<'r, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.0
    }
}
