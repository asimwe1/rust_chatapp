use std::sync::atomic::{AtomicBool, Ordering};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response, Rocket};

use super::policy::*;

/// A [`Fairing`](/rocket/fairing/trait.Fairing.html)
/// that adds HTTP headers to outgoing responses that control security features
/// on the browser.
///
/// # Usage
///
/// `SpaceHelmet` can be used in several ways.
///
/// To use it with its [defaults](#method.default), create a new instance and
/// attach it to [Rocket](/rocket/struct.Rocket.html)
/// as shown.
///
/// ```rust
/// # extern crate rocket;
/// # extern crate rocket_contrib;
/// use rocket_contrib::space_helmet::SpaceHelmet;
///
/// let rocket = rocket::ignite().attach(SpaceHelmet::default());
/// ```
///
/// To enable an additional header, call the method for that header, with the
/// policy for that header, before attach e.g.
///
/// ```rust
/// # extern crate rocket;
/// # extern crate rocket_contrib;
/// use rocket_contrib::space_helmet::{SpaceHelmet, ReferrerPolicy};
///
/// let helmet = SpaceHelmet::default().referrer_policy(ReferrerPolicy::NoReferrer);
/// let rocket = rocket::ignite().attach(helmet);
/// ```
///
/// To disable a header, call the method for that header with `None` as
/// the policy.
///
/// ```rust
/// # extern crate rocket;
/// # extern crate rocket_contrib;
/// use rocket_contrib::space_helmet::SpaceHelmet;
///
/// let helmet = SpaceHelmet::default().no_sniff(None);
/// let rocket = rocket::ignite().attach(helmet);
/// ```
///
/// `SpaceHelmet` supports the builder pattern to configure multiple policies
///
/// ```rust
/// # extern crate rocket;
/// # extern crate rocket_contrib;
/// use rocket_contrib::space_helmet::{HstsPolicy, ExpectCtPolicy, ReferrerPolicy, SpaceHelmet};
///
/// let helmet = SpaceHelmet::default()
///     .hsts(HstsPolicy::default())
///     .expect_ct(ExpectCtPolicy::default())
///     .referrer_policy(ReferrerPolicy::default());
///
/// let rocket = rocket::ignite().attach(helmet);
/// ```
///
/// # TLS and HSTS
///
/// If TLS is enabled when a [Rocket](rocket/struct.Rocket.html)
/// is [launched](/rocket/fairing/trait.Fairing.html#method.on_launch)
/// in a non-development environment e.g.  [Staging](rocket/struct.Config.html#method.staging)
/// or [Production](/rocket/struct.Config.html#method.production)
/// `SpaceHelmet` enables hsts with its default policy and issue a
/// warning.
///
/// To get rid of this warning, set an [hsts](#method.hsts) policy if you are using tls.
pub struct SpaceHelmet<'a> {
    expect_ct_policy: Option<ExpectCtPolicy<'a>>,
    no_sniff_policy: Option<NoSniffPolicy>,
    xss_protect_policy: Option<XssPolicy<'a>>,
    frameguard_policy: Option<FramePolicy<'a>>,
    hsts_policy: Option<HstsPolicy>,
    force_hsts_policy: Option<HstsPolicy>,
    force_hsts: AtomicBool,
    referrer_policy: Option<ReferrerPolicy>,
}

// helper for Helmet.apply
macro_rules! try_apply_header {
    ($self:ident, $response:ident, $policy_name:ident) => {
        if let Some(ref policy) = $self.$policy_name {
            if $response.set_header(policy) {
                warn_!(
                    "(Space Helmet): set_header failed, found existing header \"{}\"",
                    Header::from(policy).name
                );
            }
        }
    };
}

impl<'a> Default for SpaceHelmet<'a> {
    /// Returns a new `SpaceHelmet` instance. See [table](/rocket_contrib/space_helmet/index.html#what-it-supports) for
    /// a description of what policies are used by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// # extern crate rocket_contrib;
    /// use rocket_contrib::space_helmet::SpaceHelmet;
    ///
    /// let helmet = SpaceHelmet::default();
    /// ```
    fn default() -> Self {
        Self {
            expect_ct_policy: None,
            no_sniff_policy: Some(NoSniffPolicy::default()),
            frameguard_policy: Some(FramePolicy::default()),
            xss_protect_policy: Some(XssPolicy::default()),
            hsts_policy: None,
            force_hsts_policy: Some(HstsPolicy::default()),
            force_hsts: AtomicBool::new(false),
            referrer_policy: None,
        }
    }
}

impl<'a> SpaceHelmet<'a> {
    /// Same as [`SpaceHelmet::default()`].
    pub fn new() -> Self {
        SpaceHelmet::default()
    }

    /// Sets the [X-XSS-Protection](
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection)
    /// header to the given `policy` or disables it if `policy == None`.
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// # extern crate rocket_contrib;
    /// use rocket::http::uri::Uri;
    /// use rocket_contrib::space_helmet::{SpaceHelmet, XssPolicy};
    ///
    /// let report_uri = Uri::parse("https://www.google.com").unwrap();
    /// let helmet = SpaceHelmet::new().xss_protect(XssPolicy::EnableReport(report_uri));
    /// ```
    pub fn xss_protect<T: Into<Option<XssPolicy<'a>>>>(mut self, policy: T) -> SpaceHelmet<'a> {
        self.xss_protect_policy = policy.into();
        self
    }

    /// Sets the [X-Content-Type-Options](
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options)
    /// header to `policy` or disables it if `policy == None`.
    /// # Example
    ///
    /// ```rust
    ///
    /// use rocket_contrib::space_helmet::{SpaceHelmet, NoSniffPolicy};
    ///
    /// let helmet = SpaceHelmet::new().no_sniff(NoSniffPolicy::Enable);
    /// ```
    pub fn no_sniff<T: Into<Option<NoSniffPolicy>>>(mut self, policy: T) -> SpaceHelmet<'a> {
        self.no_sniff_policy = policy.into();
        self
    }

    /// Sets the [X-Frame-Options](
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options)
    /// header to `policy`, or disables it if `policy == None`.
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// # extern crate rocket_contrib;
    ///
    /// use rocket::http::uri::Uri;
    /// use rocket_contrib::space_helmet::{SpaceHelmet, FramePolicy};
    ///
    /// let allow_uri = Uri::parse("https://www.google.com").unwrap();
    /// let helmet = SpaceHelmet::new().frameguard(FramePolicy::AllowFrom(allow_uri));
    /// ```
    pub fn frameguard<T: Into<Option<FramePolicy<'a>>>>(mut self, policy: T) -> SpaceHelmet<'a> {
        self.frameguard_policy = policy.into();
        self
    }

    /// Sets the [Strict-Transport-Security](
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security)
    /// header to `policy`, or disables it if `policy == None`.
    /// # Example
    ///
    /// ```rust
    /// use rocket_contrib::space_helmet::{SpaceHelmet, HstsPolicy};
    ///
    /// let helmet = SpaceHelmet::new().hsts(HstsPolicy::default());
    /// ```
    pub fn hsts<T: Into<Option<HstsPolicy>>>(mut self, policy: T) -> SpaceHelmet<'a> {
        self.hsts_policy = policy.into();
        self
    }

    /// Sets the [Expect-CT](
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Expect-CT)
    /// header to `policy`, or disables it if `policy == None`.
    /// ```rust
    /// # extern crate rocket;
    /// # extern crate rocket_contrib;
    /// # extern crate time;
    /// use rocket::http::uri::Uri;
    /// use rocket_contrib::space_helmet::{SpaceHelmet, ExpectCtPolicy};
    /// use time::Duration;
    ///
    /// let report_uri = Uri::parse("https://www.google.com").unwrap();
    /// let helmet = SpaceHelmet::new()
    ///              .expect_ct(ExpectCtPolicy::ReportAndEnforce(
    ///                         Duration::days(30), report_uri));
    /// ```
    pub fn expect_ct<T: Into<Option<ExpectCtPolicy<'a>>>>(mut self, policy: T) -> SpaceHelmet<'a> {
        self.expect_ct_policy = policy.into();
        self
    }

    /// Sets the [Referrer-Policy](
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy)
    /// header to `policy`, or disables it if `policy == None`.
    /// ```rust
    /// # extern crate rocket;
    /// # extern crate rocket_contrib;
    ///
    /// use rocket_contrib::space_helmet::{ReferrerPolicy, SpaceHelmet};
    ///
    /// let helmet = SpaceHelmet::new().referrer_policy(ReferrerPolicy::NoReferrer);
    /// ```
    pub fn referrer_policy<T: Into<Option<ReferrerPolicy>>>(
        mut self,
        policy: T,
    ) -> SpaceHelmet<'a> {
        self.referrer_policy = policy.into();
        self
    }

    fn apply(&self, response: &mut Response) {
        try_apply_header!(self, response, no_sniff_policy);
        try_apply_header!(self, response, xss_protect_policy);
        try_apply_header!(self, response, frameguard_policy);
        try_apply_header!(self, response, expect_ct_policy);
        try_apply_header!(self, response, referrer_policy);
        if self.hsts_policy.is_some() {
            try_apply_header!(self, response, hsts_policy);
        } else {
            if self.force_hsts.load(Ordering::Relaxed) {
                try_apply_header!(self, response, force_hsts_policy);
            }
        }
    }
}

impl Fairing for SpaceHelmet<'static> {
    fn info(&self) -> Info {
        Info {
            name: "Rocket SpaceHelmet (HTTP Security Headers)",
            kind: Kind::Response | Kind::Launch,
        }
    }

    fn on_response(&self, _request: &Request, response: &mut Response) {
        self.apply(response);
    }

    fn on_launch(&self, rocket: &Rocket) {
        if rocket.config().tls_enabled()
            && !rocket.config().environment.is_dev()
            && !self.hsts_policy.is_some()
        {
            warn_!("(Space Helmet): deploying with TLS without enabling hsts.");
            warn_!("Enabling default HSTS policy.");
            info_!("To disable this warning, configure an HSTS policy.");
            self.force_hsts.store(true, Ordering::Relaxed);
        }
    }
}
