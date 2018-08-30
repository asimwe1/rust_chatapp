use std::borrow::Cow;
use rocket::http::uri::Uri;
use rocket::http::Header;

use super::time::Duration;

/// The [Referrer-Policy] header tells the browser if should send all or part of URL of the current
/// page to the next site the user navigates to via the [Referer] header. This can be important
/// for security as the URL itself might expose sensitive data, such as a hidden file path or
/// personal identifier.
///
/// [Referrer-Policy]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy
/// [Referer]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referer
pub enum ReferrerPolicy {
    /// Omits the `Referer` header (_SpaceHelmet default_).
    NoReferrer,

    /// Omits the `Referer` header on connection downgrade i.e. following HTTP link from HTTPS site
    /// (_Browser default_).
    NoReferrerWhenDowngrade,

    /// Only send the origin of part of the URL, e.g. the origin of https://foo.com/bob.html is
    /// https://foo.com
    Origin,

    /// Send full URL for same-origin requests, only send origin part when replying
    /// to [cross-origin] requests.
    ///
    /// [cross-origin]: https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS
    OriginWhenCrossOrigin,

    /// Send full URL for same-origin requests only.
    SameOrigin,

    /// Only send origin part of URL, only send if protocol security level remains the same e.g.
    /// HTTPS to HTTPS.
    StrictOrigin,

    /// Send full URL for same-origin requests. For cross-origin requests, only send origin
    /// part of URL if protocl security level remains the same e.g. HTTPS to HTTPS.
    StrictOriginWhenCrossOrigin,

    /// Send full URL for same-origin or cross-origin requests. _This will leak the full
    /// URL of TLS protected resources to insecure origins. Use with caution._
    UnsafeUrl,
 }

/// Defaults to [`ReferrerPolicy::NoReferrer`]. Tells the browser Omit the `Referer` header.
impl Default for ReferrerPolicy {
    fn default() -> ReferrerPolicy {
        ReferrerPolicy::NoReferrer
    }
}

impl<'a, 'b> From<&'a ReferrerPolicy> for Header<'b> {
    fn from(policy: &ReferrerPolicy) -> Header<'b> {
        let policy_string = match policy {
            ReferrerPolicy::NoReferrer => "no-referrer",
            ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            ReferrerPolicy::Origin => "origin",
            ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
            ReferrerPolicy::SameOrigin => "same-origin",
            ReferrerPolicy::StrictOrigin => "strict-origin",
            ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                "strict-origin-when-cross-origin"
            }
            ReferrerPolicy::UnsafeUrl => "unsafe-url",
        };

        Header::new("Referrer-Policy", policy_string)
    }
}


/// The [Expect-CT] header tells browser to enable [Certificate Transparency] checking, which
/// can detect and prevent the misuse of the site's certificate. Read all [`ExpectCtPolicy`]
/// documentation before use.
///
/// [Certificate Transparency]
/// solves a variety of problems with public TLS/SSL certificate management and is valuable
/// measure if your standing up a public website. If your [getting started] with certificate
/// transparency, be sure that your [site is in compliance][getting started] before you turn on
/// enforcement with [`ExpectCtPolicy::Enforce`] or [`ExpectCtPolicy::ReportAndEnforce`]
/// otherwise the browser will stop talking to your site until you bring it into compliance
/// or [`Duration`] time elapses. _You have been warned_.
///
///
///
/// [Expect-CT]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Expect-CT
/// [Certificate Transparency]: http://www.certificate-transparency.org/what-is-ct
/// [getting started]: http://www.certificate-transparency.org/getting-started
pub enum ExpectCtPolicy<'a> {
    /// Tells browser to enforce certificate compliance for [`Duration`] seconds.
    /// Check if your site is in compliance before turning on enforcement.
    /// (_SpaceHelmet_ default).
    Enforce(Duration),

    /// Tells browser to report compliance violations certificate transparency for [`Duration`]
    /// seconds. Doesn't provide any protection but is a good way make sure
    /// things are working correctly before turning on enforcement in production.
    Report(Duration, Uri<'a>),

    /// Enforces compliance and supports notification to if there has been a violation for
    /// [`Duration`].
    ReportAndEnforce(Duration, Uri<'a>),
}

/// Defaults to [`ExpectCtPolicy::Enforce(Duration::days(30))`], enforce CT
/// compliance, see [draft] standard for more.
///
/// [draft]: https://tools.ietf.org/html/draft-ietf-httpbis-expect-ct-03#page-15
impl<'a> Default for ExpectCtPolicy<'a> {
    fn default() -> ExpectCtPolicy<'a> {
        ExpectCtPolicy::Enforce(Duration::days(30))
    }
}

impl<'a, 'b> From<&'a ExpectCtPolicy<'a>> for Header<'b> {
    fn from(policy: &ExpectCtPolicy<'a>) -> Header<'b> {
        let policy_string =  match policy {
            ExpectCtPolicy::Enforce(max_age) => format!("max-age={}, enforce", max_age.num_seconds()),
            ExpectCtPolicy::Report(max_age, url) => {
                format!("max-age={}, report-uri=\"{}\"", max_age.num_seconds(), url)
            }
            ExpectCtPolicy::ReportAndEnforce(max_age, url) => {
                format!("max-age={}, enforce, report-uri=\"{}\"", max_age.num_seconds(), url)
            }
        };

        Header::new("Expect-CT", policy_string)
    }
}

/// The [X-Content-Type-Options] header can tell the browser to turn off [mime sniffing](
/// https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types#MIME_sniffing) which
/// can prevent certain [attacks].
///
/// [X-Content-Type-Options]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options
/// [attacks]: https://helmetjs.github.io/docs/dont-sniff-mimetype/
pub enum NoSniffPolicy {

    ///Turns off mime sniffing.
    Enable,
}

/// Defaults to [`NoSniffPolicy::Enable`], turns off mime sniffing.
impl Default for NoSniffPolicy {
    fn default() -> NoSniffPolicy {
        NoSniffPolicy::Enable
    }
}

impl<'a, 'b> From<&'a NoSniffPolicy> for Header<'b> {
    fn from(_policy: &NoSniffPolicy) -> Header<'b> {
        Header::new("X-Content-Type-Options", "nosniff")
    }
}

/// The HTTP [Strict-Transport-Security] (HSTS) header tells the browser that the site should only
/// be accessed using HTTPS instead of HTTP. HSTS prevents a variety of downgrading attacks and
/// should always be used when TLS is enabled.  `SpaceHelmet` will turn HSTS on and
/// issue a warning if you enable TLS without enabling HSTS in [Staging ] or [Production].
/// Read full [`HstsPolicy`] documentation before you configure this.
///
/// HSTS is important for HTTPS security, however, incorrectly configured HSTS can lead to problems as
/// you are disallowing access to non-HTTPS enabled parts of your site. [Yelp engineering] has good
/// discussion of potential challenges that can arise, and how to roll this out in a large scale setting.
/// So, if you use TLS, use HSTS, just roll it out with care.
///
/// Finally, requiring TLS use with valid certificates may be something of a nuisance in
/// development settings, so you may with to restrict HSTS use to [Staging] and [Production].
///
/// [TLS]: https://rocket.rs/guide/configuration/#configuring-tls
/// [Strict-Transport-Security]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security
/// [default policy]: /rocket_contrib/space_helmet/enum.HstsPolicy.html#impl-Default
/// [Yelp engineering]: https://engineeringblog.yelp.com/2017/09/the-road-to-hsts.html
/// [Staging]: /rocket/config/enum.Environment.html#variant.Staging
/// [Production]: /rocket/config/enum.Environment.html#variant.Production
pub enum HstsPolicy {
    /// Browser should only permit this site to be accesses by HTTPS for the next [`Duration`]
    /// seconds.
    Enable(Duration),

    /// Same as above, but also apply to all of the sites subdomains.
    IncludeSubDomains(Duration),

    /// Google maintains an [HSTS preload service] that can be used to prevent
    /// the browser from ever connecting to your site over an insecure connection.
    /// Read more [here]. Don't enable this before you have registered your site
    ///
    /// [HSTS preload service]: https://hstspreload.org/
    /// [here]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security#Preloading_Strict_Transport_Security
    Preload(Duration),
}

/// Defaults to `HstsPolicy::Enable(Duration::weeks(52))`.
impl Default for HstsPolicy {
    fn default() -> HstsPolicy {
        HstsPolicy::Enable(Duration::weeks(52))
    }
}

impl<'a, 'b> From<&'a HstsPolicy> for Header<'b> {
    fn from(policy: &HstsPolicy) -> Header<'b> {
        let policy_string = match policy {
            HstsPolicy::Enable(max_age) => format!("max-age={}", max_age.num_seconds()),
            HstsPolicy::IncludeSubDomains(max_age) => {
                format!("max-age={}; includeSubDomains", max_age.num_seconds())
            }
            HstsPolicy::Preload(max_age) => format!("max-age={}; preload", max_age.num_seconds()),
        };

        Header::new("Strict-Transport-Security", policy_string)
    }
}

/// The [X-Frame-Options] header controls whether the browser should allow the page
/// to render in a `<frame>`, [`<iframe>`][iframe] or `<object>`. This can be used
/// to prevent [clickjacking] attacks.
///
///
/// [X-Frame-Options]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options
/// [clickjacking]: https://en.wikipedia.org/wiki/Clickjacking
/// [owasp-clickjacking]: https://www.owasp.org/index.php/Clickjacking_Defense_Cheat_Sheet
/// [iframe]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe
pub enum FramePolicy<'a> {
    /// Page cannot be displayed in a frame.
    Deny,

    /// Page can only be displayed in a frame if the page trying to render it came from
    /// the same-origin. Interpretation of same-origin is [browser dependant][X-Frame-Options].
    ///
    /// [X-Frame-Options]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options
    SameOrigin,

    /// Page can only be displayed in a frame if the page trying to render it came from
    /// the origin given `Uri`. Interpretation of origin is [browser dependant][X-Frame-Options].
    ///
    /// [X-Frame-Options]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options
    AllowFrom(Uri<'a>),
}

///Defaults to [`FramePolicy::SameOrigin`].
impl<'a> Default for FramePolicy<'a> {
    fn default() -> FramePolicy<'a> {
        FramePolicy::SameOrigin
    }
}

impl<'a, 'b> From<&'a FramePolicy<'a>> for Header<'b> {
    fn from(policy: &FramePolicy<'a>) -> Header<'b> {
        let policy_string: Cow<'static, str> = match policy {
            FramePolicy::Deny => "DENY".into(),
            FramePolicy::SameOrigin => "SAMEORIGIN".into(),
            FramePolicy::AllowFrom(uri) => format!("ALLOW-FROM {}", uri).into(),
        };

        Header::new("X-Frame-Options", policy_string)
    }
}

/// The [X-XSS-Protection] header tells the browsers to filter some forms of reflected
/// XSS([cross-site scripting]) attacks.
///
/// [X-XSS-Protection]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection
/// [cross-site scripting]: https://developer.mozilla.org/en-US/docs/Glossary/Cross-site_scripting
pub enum XssPolicy<'a> {
    /// Disables XSS filtering.
    Disable,

    /// Enables XSS filtering, if XSS detected browser will sanitize and render page (_often browser
    /// default_).
    Enable,

    /// Enables XSS filtering, if XSS detected browser will block rendering of page (_SpaceHelmet default_).
    EnableBlock,

    /// Enables XSS filtering, if XSS detected browser will sanitize and render page and report the
    /// violation to the given `Uri`. (Chromium only)
    EnableReport(Uri<'a>),
}

/// Defaults to [`XssPolicy::EnableBlock`], turns on XSS filtering and blocks page rendering if
/// detected.
impl<'a> Default for XssPolicy<'a> {
    fn default() -> XssPolicy<'a> {
        XssPolicy::EnableBlock
    }
}

impl<'a, 'b> From<&'a XssPolicy<'a>> for Header<'b> {
    fn from(policy: &XssPolicy) -> Header<'b> {
        let policy_string: Cow<'static, str> = match policy {
            XssPolicy::Disable => "0".into(),
            XssPolicy::Enable => "1".into(),
            XssPolicy::EnableBlock => "1; mode=block".into(),
            XssPolicy::EnableReport(u) => format!("{}{}", "1; report=", u).into(),
        };

        Header::new("X-XSS-Protection", policy_string)
    }
}
