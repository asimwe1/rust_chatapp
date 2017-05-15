use {Rocket, Request, Response, Data};
use fairing::{Fairing, Kind};

#[derive(Default)]
pub struct Fairings {
    all_fairings: Vec<Box<Fairing>>,
    launch: Vec<&'static Fairing>,
    request: Vec<&'static Fairing>,
    response: Vec<&'static Fairing>,
}

impl Fairings {
    #[inline]
    pub fn new() -> Fairings {
        Fairings::default()
    }

    #[inline]
    pub fn attach(&mut self, fairing: Box<Fairing>) {
        // Get the kind information.
        let kind = fairing.info().kind;

        // The `Fairings` structure separates `all_fairings` into kind groups so
        // we don't have to search through all fairings and do a comparison at
        // runtime. We need references since a single structure can be multiple
        // kinds. The lifetime of that reference is really the lifetime of the
        // `Box` for referred fairing, but that lifetime is dynamic; there's no
        // way to express it. So we cheat and say that the lifetime is
        // `'static` and cast it here. For this to be safe, the following must
        // be preserved:
        //
        //  1) The references can never be exposed with a `'static` lifetime.
        //  2) The `Box<Fairing>` must live for the lifetime of the reference.
        //
        // We maintain these invariants by not exposing the references and never
        // deallocating `Box<Fairing>` structures. As such, the references will
        // always be valid. Note: `ptr` doesn't point into the `Vec`, so
        // reallocations there are irrelvant. Instead, it points into the heap.
        let ptr: &'static Fairing = unsafe { ::std::mem::transmute(&*fairing) };

        self.all_fairings.push(fairing);
        if kind.is(Kind::Launch) { self.launch.push(ptr); }
        if kind.is(Kind::Request) { self.request.push(ptr); }
        if kind.is(Kind::Response) { self.response.push(ptr); }
    }

    #[inline(always)]
    pub fn handle_launch(&mut self, mut rocket: Rocket) -> Option<Rocket> {
        let mut success = Some(());
        for f in &self.launch {
            rocket = f.on_launch(rocket).unwrap_or_else(|r| { success = None; r });
        }

        success.map(|_| rocket)
    }

    #[inline(always)]
    pub fn handle_request(&self, req: &mut Request, data: &Data) {
        for fairing in &self.request {
            fairing.on_request(req, data);
        }
    }

    #[inline(always)]
    pub fn handle_response(&self, request: &Request, response: &mut Response) {
        for fairing in &self.response {
            fairing.on_response(request, response);
        }
    }

    pub fn pretty_print_counts(&self) {
        use term_painter::ToStyle;
        use term_painter::Color::{White, Magenta};

        if self.all_fairings.len() > 0 {
            info!("📡  {}:", Magenta.paint("Fairings"));
        }

        fn info_if_nonempty(kind: &str, fairings: &[&Fairing]) {
            let names: Vec<&str> = fairings.iter().map(|f| f.info().name).collect();
            info_!("{} {}: {}", White.paint(fairings.len()), kind,
                White.paint(names.join(", ")));
        }

        info_if_nonempty("launch", &self.launch);
        info_if_nonempty("request", &self.request);
        info_if_nonempty("response", &self.response);
    }
}
