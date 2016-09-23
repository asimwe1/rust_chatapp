#[macro_export]
macro_rules! engine_set {
    ($($feature:expr => $engine:ident),+,) => ({
        use std::collections::HashSet;
        let mut set = HashSet::new();
        $(
            #[cfg(feature = $feature)]
            fn $engine(set: &mut HashSet<String>) {
                set.insert($engine::EXT.to_string());
            }

            #[cfg(not(feature = $feature))]
            fn $engine(_: &mut HashSet<String>) {  }

            $engine(&mut set);
        )+
        set
    });
}

#[macro_export]
macro_rules! render_set {
    ($name:expr, $info:expr, $ctxt:expr, $($feature:expr => $engine:ident),+,) => ({$(
        #[cfg(feature = $feature)]
        fn $engine<T: Serialize>(name: &str, info: &TemplateInfo, c: &T)
                -> Option<Template> {
            if info.extension == $engine::EXT {
                let rendered = $engine::render(name, info, c);
                return Some(Template(rendered, info.data_type.clone()));
            }

            None
        }

        #[cfg(not(feature = $feature))]
        fn $engine<T: Serialize>(_: &str, _: &TemplateInfo, _: &T)
                -> Option<Template> { None }

        if let Some(template) = $engine($name, &$info, &$ctxt) {
            return template
        }
    )+});
}

