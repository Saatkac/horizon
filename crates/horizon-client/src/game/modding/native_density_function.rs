//! Native density function adapter: the mods' `DensityFunctionType`s as the engine's.
use horizon_world::worldgen::mod_density_functions::{
    ModDensityFunction, ModDensityFunctionType, ModDensityFunctionTypes,
    install_mod_density_function_types,
};
use mod_api::{DensityFunction, DensityFunctionType};
use std::sync::Arc;

struct TypeAdapter(Arc<dyn DensityFunctionType>);
impl ModDensityFunctionType for TypeAdapter {
    fn decode(&self, json: &str) -> Result<Arc<dyn ModDensityFunction>, String> {
        let function = self.0.decode(json)?;
        Ok(Arc::new(FunctionAdapter(function)))
    }
}

struct FunctionAdapter(Arc<dyn DensityFunction>);
impl ModDensityFunction for FunctionAdapter {
    fn range(&self) -> [f32; 2] {
        self.0.range()
    }
    fn varies_with_y(&self) -> bool {
        self.0.varies_with_y()
    }
    fn sample(&self, position: [i32; 3], seed: i64) -> f32 {
        self.0.sample(position, seed)
    }
    fn fill(
        &self,
        origin: [i32; 3],
        step: [i32; 3],
        count: [usize; 3],
        seed: i64,
        out: &mut [f32],
    ) {
        self.0.fill(origin, step, count, seed, out);
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
        assert_eq!(tilt.sample([3, 70, -4], 5), 11.0);
        let mut out = [0.0; 4];
        tilt.fill([-1, 0, 0], [4, 8, 1], [2, 2, 1], 0, &mut out);
        assert_eq!(out, [-2.0, -2.0, 6.0, 6.0]);
        let refused = types["test:tilt"].decode(r#"{"type":"test:tilt"}"#);
        assert_eq!(refused.err().as_deref(), Some("no slope"));
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
