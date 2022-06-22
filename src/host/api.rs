//! Lunatic VM host functions.

pub mod error {
    wit_bindgen_rust::import!("wit/lunatic_error.wit");

    #[doc(inline)]
    pub use lunatic_error::*;
}

pub mod message {
    wit_bindgen_rust::import!("wit/lunatic_message.wit");

    #[doc(inline)]
    pub use lunatic_message::*;
}

pub mod timer {
    wit_bindgen_rust::import!("wit/lunatic_timer.wit");

    #[doc(inline)]
    pub use lunatic_timer::*;
}

pub mod networking {
    wit_bindgen_rust::import!("wit/lunatic_networking.wit");

    #[doc(inline)]
    pub use lunatic_networking::*;
}

pub mod process {
    wit_bindgen_rust::import!("wit/lunatic_process.wit");

    #[doc(inline)]
    pub use lunatic_process::*;
}

pub mod registry {
    wit_bindgen_rust::import!("wit/lunatic_registry.wit");

    #[doc(inline)]
    pub use lunatic_registry::*;
}

pub mod wasi {
    wit_bindgen_rust::import!("wit/lunatic_wasi.wit");

    #[doc(inline)]
    pub use lunatic_wasi::*;
}

pub mod version {
    wit_bindgen_rust::import!("wit/lunatic_version.wit");

    #[doc(inline)]
    pub use lunatic_version::*;
}
