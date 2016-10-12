//! Types and traits to handle form processing.
//!
//! In general, you will deal with forms in Rocket via the `form` parameter in
//! routes:
//!
//! ```rust,ignore
//! #[post("/", form = <my_form>)]
//! fn form_submit(my_form: MyType) -> ...
//! ```
//!
//! Form parameter types must implement the [FromForm](trait.FromForm.html)
//! trait, which is automatically derivable. Automatically deriving `FromForm`
//! for a structure requires that all of its fields implement
//! [FromFormValue](trait.FormFormValue.html). See the
//! [codegen](/rocket_codegen/) documentation or the [forms guide](/guide/forms)
//! for more information on forms and on deriving `FromForm`.

mod form_items;
mod from_form;
mod from_form_value;

pub use self::form_items::FormItems;
pub use self::from_form::FromForm;
pub use self::from_form_value::FromFormValue;

use std::marker::PhantomData;
use std::fmt::{self, Debug};
use std::io::Read;

use request::{Request, FromData, Data, DataOutcome};

// This works, and it's safe, but it sucks to have the lifetime appear twice.
pub struct Form<'f, T: FromForm<'f> + 'f> {
    object: T,
    form_string: String,
    _phantom: PhantomData<&'f T>,
}

impl<'f, T: FromForm<'f> + 'f> Form<'f, T> {
    pub fn get(&'f self) -> &'f T {
        &self.object
    }

    pub fn get_mut(&'f mut self) -> &'f mut T {
        &mut self.object
    }

    pub fn raw_form_string(&self) -> &str {
        &self.form_string
    }

    // Alright, so here's what's going on here. We'd like to have form
    // objects have pointers directly to the form string. This means that
    // the form string has to live at least as long as the form object. So,
    // to enforce this, we store the form_string along with the form object.
    //
    // So far so good. Now, this means that the form_string can never be
    // deallocated while the object is alive. That implies that the
    // `form_string` value should never be moved away. We can enforce that
    // easily by 1) not making `form_string` public, and 2) not exposing any
    // `&mut self` methods that could modify `form_string`.
    //
    // Okay, we do all of these things. Now, we still need to give a
    // lifetime to `FromForm`. Which one do we choose? The danger is that
    // references inside `object` may be copied out, and we have to ensure
    // that they don't outlive this structure. So we would really like
    // something like `self` and then to transmute to that. But this doesn't
    // exist. So we do the next best: we use the first lifetime supplied by the
    // caller via `get()` and contrain everything to that lifetime. This is, in
    // reality a little coarser than necessary, but the user can simply move the
    // call to right after the creation of a Form object to get the same effect.
    fn new(form_string: String) -> Result<Self, (String, T::Error)> {
        let long_lived_string: &'f str = unsafe {
            ::std::mem::transmute(form_string.as_str())
        };

        match T::from_form_string(long_lived_string) {
            Ok(obj) => Ok(Form {
                form_string: form_string,
                object: obj,
                _phantom: PhantomData
            }),
            Err(e) => Err((form_string, e))
        }
    }
}

impl<'f, T: FromForm<'f> + 'static> Form<'f, T> {
    pub fn into_inner(self) -> T {
        self.object
    }
}

impl<'f, T: FromForm<'f> + Debug + 'f> Debug for Form<'f, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} from form string: {:?}", self.object, self.form_string)
    }
}

impl<'f, T: FromForm<'f>> FromData for Form<'f, T> where T::Error: Debug {
    type Error = Option<String>;

    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error> {
        if !request.content_type().is_form() {
            warn_!("Form data does not have form content type.");
            return DataOutcome::Forward(data);
        }

        let mut form_string = String::with_capacity(4096);
        let mut stream = data.open().take(32768);
        if let Err(e) = stream.read_to_string(&mut form_string) {
            error_!("IO Error: {:?}", e);
            DataOutcome::Failure(None)
        } else {
            match Form::new(form_string) {
                Ok(form) => DataOutcome::Success(form),
                Err((form_string, e)) => {
                    error_!("Failed to parse value from form: {:?}", e);
                    DataOutcome::Failure(Some(form_string))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Form;
    use ::request::FromForm;

    struct Simple<'s> {
        value: &'s str
    }

    struct Other {
        value: String
    }

    impl<'s> FromForm<'s> for Simple<'s> {
        type Error = &'s str;

        fn from_form_string(fs: &'s str) -> Result<Simple<'s>, &'s str> {
            Ok(Simple { value: fs })
        }
    }

    impl<'s> FromForm<'s> for Other {
        type Error = &'s str;

        fn from_form_string(fs: &'s str) -> Result<Other, &'s str> {
            Ok(Other { value: fs.to_string() })
        }
    }

    #[test]
    fn test_lifetime() {
        let form_string = "hello=world".to_string();
        let form: Form<Simple> = Form::new(form_string).unwrap();

        let string: &str = form.get().value;
        assert_eq!(string, "hello=world");
    }

    #[test]
    fn test_lifetime_2() {
        let form_string = "hello=world".to_string();
        let mut _y = "hi";
        let _form: Form<Simple> = Form::new(form_string).unwrap();
        // _y = form.get().value;

        // fn should_not_compile<'f>(form: Form<'f, &'f str>) -> &'f str {
        //     form.get()
        // }

        // fn should_not_compile_2<'f>(form: Form<'f, Simple<'f>>) -> &'f str {
        //     form.into_inner().value
        // }

        // assert_eq!(should_not_compile(form), "hello=world");
    }

    #[test]
    fn test_lifetime_3() {
        let form_string = "hello=world".to_string();
        let form: Form<Other> = Form::new(form_string).unwrap();

        // Not bad.
        fn should_compile(form: Form<Other>) -> String {
            form.into_inner().value
        }

        assert_eq!(should_compile(form), "hello=world".to_string());
    }

    #[test]
    fn test_lifetime_4() {
        let form_string = "hello=world".to_string();
        let form: Form<Simple> = Form::new(form_string).unwrap();

        fn should_compile<'f>(_form: Form<'f, Simple<'f>>) {  }

        should_compile(form)
        // assert_eq!(should_not_compile(form), "hello=world");
    }
}

