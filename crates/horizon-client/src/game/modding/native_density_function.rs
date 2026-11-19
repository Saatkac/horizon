//! Native density function adapter: the mods' `DensityFunctionType`s as the engine's.
use horizon_world::worldgen::mod_density_functions::{
    ModDensityFunction, ModDensityFunctionType, ModDensityFunctionTypes,
    install_mod_density_function_types,
};
use mod_api::{DensityFunction, DensityFunctionType};
use std::sync::Arc;

/// A mod's type. Every call into the mod's code is contained: a panic comes back as an error,
/// which the engine reports with the type's id, instead of ending the game from a worldgen worker.
struct TypeAdapter(Arc<dyn DensityFunctionType>);
impl ModDensityFunctionType for TypeAdapter {
    fn decode(&self, json: &str) -> Result<Arc<dyn ModDensityFunction>, String> {
        let decoded = horizon_client::crash::contain(|| {
            let function = self.0.decode(json)?;
            // Asked once, here, so that the engine never calls them again.
            let range = function.range();
            let varies_with_y = function.varies_with_y();
            Ok::<_, String>(FunctionAdapter {
                function,
                range,
                varies_with_y,
            })
        });
        let adapter = decoded.map_err(|panic| format!("decoding panicked: {panic}"))??;
        Ok(Arc::new(adapter))
    }
}

struct FunctionAdapter {
    function: Arc<dyn DensityFunction>,
    range: [f32; 2],
    varies_with_y: bool,
}
impl ModDensityFunction for FunctionAdapter {
    fn range(&self) -> [f32; 2] {
        self.range
    }
    fn varies_with_y(&self) -> bool {
        self.varies_with_y
    }
    fn sample(&self, position: [i32; 3], seed: i64) -> Result<f32, String> {
        horizon_client::crash::contain(|| self.function.sample(position, seed))
            .map_err(|panic| format!("sample panicked: {panic}"))
    }
    fn fill(
        &self,
        origin: [i32; 3],
        step: [i32; 3],
        count: [usize; 3],
        seed: i64,
        out: &mut [f32],
    ) -> Result<(), String> {
        horizon_client::crash::contain(|| self.function.fill(origin, step, count, seed, out))
            .map_err(|panic| format!("fill panicked: {panic}"))
    }
}

fn installed_density_function_types() -> ModDensityFunctionTypes {
    mod_api::__private::density_function_type_declarations()
        .into_iter()
        .map(|(id, density_type)| {
            (
                id,
                Arc::new(TypeAdapter(density_type)) as Arc<dyn ModDensityFunctionType>,
            )
        })
        .collect()
}

/// Hand the density function types the mods registered to the engine, before any world opens.
pub(in crate::game) fn install() {
    if install_mod_density_function_types(installed_density_function_types()).is_err() {
        log::warn!("Mod density function types already installed; keeping the first table");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::api_values::install_parts;
    use mod_api::{Hooks, ModEntry, Registrar, RegistrationContext};

    /// `{"type": "test:tilt", "slope": s}`: s times x plus the seed, only the default `fill`.
    struct Tilt;
    struct Tilted(f32);
    impl DensityFunctionType for Tilt {
        fn decode(&self, json: &str) -> Result<Arc<dyn DensityFunction>, String> {
            let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
            let slope = value["slope"].as_f64().ok_or("no slope")?;
            Ok(Arc::new(Tilted(slope as f32)))
        }
    }
    impl DensityFunction for Tilted {
        fn range(&self) -> [f32; 2] {
            [-1e6, 1e6]
        }
        fn varies_with_y(&self) -> bool {
            false
        }
        fn sample(&self, [x, _, _]: [i32; 3], seed: i64) -> f32 {
            self.0 * x as f32 + seed as f32
        }
    }

    /// The engine reads the mod's own decoder under its namespaced id, and the decoded function
    /// keeps what the mod said; the default `fill` walks y fastest, then x, then z.
    #[test]
    fn a_registered_type_reaches_the_engine_and_decodes_its_fields() {
        struct Registers;
        #[mod_api::__subscriptions]
        impl Hooks for Registers {
            fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
                r.register_density_function_type("tilt", Tilt);
            }
        }
        install_parts(&[ModEntry::new("test", Box::new(Registers))]).unwrap();
        let types = installed_density_function_types();
        let tilt = types["test:tilt"]
            .decode(r#"{"type":"test:tilt","slope":2}"#)
            .unwrap();
        assert_eq!(tilt.range(), [-1e6, 1e6]);
        assert!(!tilt.varies_with_y());
        assert_eq!(tilt.sample([3, 70, -4], 5), Ok(11.0));
        let mut out = [0.0; 4];
        tilt.fill([-1, 0, 0], [4, 8, 1], [2, 2, 1], 0, &mut out)
            .unwrap();
        assert_eq!(out, [-2.0, -2.0, 6.0, 6.0]);
        let refused = types["test:tilt"].decode(r#"{"type":"test:tilt"}"#);
        assert_eq!(refused.err().as_deref(), Some("no slope"));
    }

    /// `{"type": "test:breaks", "in": "decode" | "range" | "fill"}`: panics there.
    struct Breaks;
    struct Broken;
    impl DensityFunctionType for Breaks {
        fn decode(&self, json: &str) -> Result<Arc<dyn DensityFunction>, String> {
            let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
            match value["in"].as_str() {
                Some("decode") => panic!("bad plates"),
                Some("range") => Ok(Arc::new(BrokenRange)),
                _ => Ok(Arc::new(Broken)),
            }
        }
    }
    impl DensityFunction for Broken {
        fn range(&self) -> [f32; 2] {
            [0.0, 1.0]
        }
        fn sample(&self, _: [i32; 3], _: i64) -> f32 {
            panic!("bad sample")
        }
        fn fill(&self, _: [i32; 3], _: [i32; 3], _: [usize; 3], _: i64, _: &mut [f32]) {
            panic!("bad fill")
        }
    }
    struct BrokenRange;
    impl DensityFunction for BrokenRange {
        fn range(&self) -> [f32; 2] {
            panic!("bad range")
        }
        fn sample(&self, _: [i32; 3], _: i64) -> f32 {
            0.0
        }
    }

    /// A panic in the mod's code comes back as an error with its message, on this thread: the
    /// decoder's turns into a load error naming the type, `fill`'s into a failed chunk.
    #[test]
    fn a_panicking_mod_function_is_an_error_not_a_crash() {
        let breaks = TypeAdapter(Arc::new(Breaks));
        let decoded = breaks.decode(r#"{"in":"decode"}"#);
        let error = decoded.err().expect("decode refused");
        assert!(
            error.starts_with("decoding panicked:") && error.contains("bad plates"),
            "{error}"
        );
        let error = breaks
            .decode(r#"{"in":"range"}"#)
            .err()
            .expect("range refused");
        assert!(error.contains("bad range"), "{error}");
        let broken = breaks.decode(r#"{"in":"fill"}"#).unwrap();
        let error = broken.sample([0, 0, 0], 0).unwrap_err();
        assert!(
            error.starts_with("sample panicked:") && error.contains("bad sample"),
            "{error}"
        );
        let error = broken
            .fill([0; 3], [1; 3], [1; 3], 0, &mut [0.0])
            .unwrap_err();
        assert!(
            error.starts_with("fill panicked:") && error.contains("bad fill"),
            "{error}"
        );
    }

    /// Each type a mod may not register is refused at startup, naming the mod.
    #[test]
    fn invalid_density_function_types_refuse_registration() {
        struct Invalid;
        #[mod_api::__subscriptions]
        impl Hooks for Invalid {
            fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
                r.register_density_function_type("tilt", Tilt);
                r.register_density_function_type("tilt", Tilt);
                r.register_density_function_type("other:tilt", Tilt);
                r.register_density_function_type("Tilt", Tilt);
            }
        }
        let errors = install_parts(&[ModEntry::new("test", Box::new(Invalid))]).unwrap_err();
        let expected = [
            "test registered density function type test:tilt twice",
            "test declares \"other:tilt\" in unowned namespace \"other\"",
            "test declares invalid identifier \"test:Tilt\"",
        ];
        assert_eq!(errors.len(), expected.len(), "{errors:?}");
        for (error, expected) in errors.iter().zip(expected) {
            assert!(error.contains(expected), "{error}");
        }
    }

    /// A wasm mod's registrar has only the operations that cross to wasm; its registration of a
    /// density function type stops loading, naming the type.
    #[test]
    #[should_panic(
        expected = "density function type \"plates\": a wasm mod cannot register a density function type"
    )]
    fn a_wasm_mods_density_function_type_is_refused_by_name() {
        struct Guest;
        impl mod_api::DimensionsRegistrar for Guest {
            fn add_dimension(&mut self, _: &str, _: mod_api::DimensionSettings) {}
        }
        mod_api::DimensionsRegistrar::add_density_function_type(
            &mut Guest,
            "plates",
            Arc::new(Tilt),
        );
    }
}
