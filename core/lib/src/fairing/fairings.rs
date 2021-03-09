use crate::{Rocket, Request, Response, Data};
use crate::fairing::{Fairing, Info, Kind};
use crate::logger::PaintExt;

use yansi::Paint;

#[derive(Default)]
pub struct Fairings {
    all_fairings: Vec<Box<dyn Fairing>>,
    attach_failures: Vec<Info>,
    // The vectors below hold indices into `all_fairings`.
    attach: Vec<usize>,
    launch: Vec<usize>,
    request: Vec<usize>,
    response: Vec<usize>,
}

impl Fairings {
    #[inline]
    pub fn new() -> Fairings {
        Fairings::default()
    }

    pub async fn attach(&mut self, fairing: Box<dyn Fairing>, mut rocket: Rocket) -> Rocket {
        // Run the `on_attach` callback if this is an 'attach' fairing.
        let kind = fairing.info().kind;
        let fairing = self.add(fairing);
        if kind.is(Kind::Attach) {
            let info = fairing.info();
            rocket = fairing.on_attach(rocket).await
                .unwrap_or_else(|r| { self.attach_failures.push(info); r })
        }

        rocket
    }

    fn add(&mut self, fairing: Box<dyn Fairing>) -> &dyn Fairing {
        let kind = fairing.info().kind;
        let index = self.all_fairings.len();
        self.all_fairings.push(fairing);

        if kind.is(Kind::Attach) { self.attach.push(index); }
        if kind.is(Kind::Launch) { self.launch.push(index); }
        if kind.is(Kind::Request) { self.request.push(index); }
        if kind.is(Kind::Response) { self.response.push(index); }

        &*self.all_fairings[index]
    }

    #[inline(always)]
    fn fairings(&self, kind: Kind) -> impl Iterator<Item = &dyn Fairing> {
        let indices = match kind {
            k if k.is(Kind::Attach) => &self.attach,
            k if k.is(Kind::Launch) => &self.launch,
            k if k.is(Kind::Request) => &self.request,
            _ => &self.response,
        };

        indices.iter().map(move |i| &*self.all_fairings[*i])
    }

    pub fn append(&mut self, others: Fairings) {
        for fairing in others.all_fairings {
            self.add(fairing);
        }
    }

    #[inline(always)]
    pub fn handle_launch(&self, rocket: &Rocket) {
        for fairing in self.fairings(Kind::Launch) {
            fairing.on_launch(rocket);
        }
    }

    #[inline(always)]
    pub async fn handle_request(&self, req: &mut Request<'_>, data: &mut Data) {
        for fairing in self.fairings(Kind::Request) {
            fairing.on_request(req, data).await
        }
    }

    #[inline(always)]
    pub async fn handle_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        for fairing in self.fairings(Kind::Response) {
            fairing.on_response(request, response).await;
        }
    }

    pub fn failures(&self) -> Option<&[Info]> {
        if self.attach_failures.is_empty() {
            None
        } else {
            Some(&self.attach_failures)
        }
    }

    pub fn pretty_print_counts(&self) {
        fn pretty_print(f: &Fairings, prefix: &str, kind: Kind) {
            let names: Vec<_> = f.fairings(kind).map(|f| f.info().name).collect();
            let num = names.len();
            let joined = names.join(", ");
            info_!("{} {}: {}", Paint::default(num).bold(), prefix, Paint::default(joined).bold());
        }

        if !self.all_fairings.is_empty() {
            info!("{}{}:", Paint::emoji("📡 "), Paint::magenta("Fairings"));
            pretty_print(self, "attach", Kind::Attach);
            pretty_print(self, "launch", Kind::Launch);
            pretty_print(self, "request", Kind::Request);
            pretty_print(self, "response", Kind::Response);
        }
    }
}

impl std::fmt::Debug for Fairings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn debug_info(fs: &Fairings, kind: Kind) -> Vec<Info> {
            fs.fairings(kind).map(|f| f.info()).collect()
        }

        f.debug_struct("Fairings")
            .field("attach", &debug_info(self, Kind::Attach))
            .field("launch", &debug_info(self, Kind::Launch))
            .field("request", &debug_info(self, Kind::Request))
            .field("response", &debug_info(self, Kind::Response))
            .finish()
    }
}
