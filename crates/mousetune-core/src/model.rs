// SPDX-License-Identifier: GPL-3.0-or-later
//! The supported DeathAdder Essential models. Every other Razer product,
//! including other DeathAdder variants, is deliberately unsupported.

use crate::types::Led;

/// USB vendor id of Razer.
pub const VENDOR_ID: u16 = 0x1532;

/// A supported mouse model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Model {
    /// DeathAdder Essential (`0x006E`): logo and scroll LEDs.
    Essential,
    /// DeathAdder Essential, White Edition (`0x0071`): logo and scroll LEDs.
    EssentialWhite,
    /// DeathAdder Essential (2021) (`0x0098`): logo LED only.
    Essential2021,
}

impl Model {
    /// Every supported model.
    pub const ALL: [Self; 3] = [Self::Essential, Self::EssentialWhite, Self::Essential2021];

    /// Looks up a model by USB product id, returning `None` for anything
    /// that is not a supported DeathAdder Essential.
    #[must_use]
    pub fn from_product_id(product_id: u16) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|model| model.product_id() == product_id)
    }

    /// USB product id.
    #[must_use]
    pub fn product_id(self) -> u16 {
        match self {
            Self::Essential => 0x006E,
            Self::EssentialWhite => 0x0071,
            Self::Essential2021 => 0x0098,
        }
    }

    /// Human readable model name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Essential => "DeathAdder Essential",
            Self::EssentialWhite => "DeathAdder Essential (White Edition)",
            Self::Essential2021 => "DeathAdder Essential (2021)",
        }
    }

    /// The LEDs this model has.
    #[must_use]
    pub fn leds(self) -> &'static [Led] {
        match self {
            Self::Essential | Self::EssentialWhite => &[Led::Logo, Led::Scroll],
            Self::Essential2021 => &[Led::Logo],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_ids_map_to_models_and_back() {
        for model in Model::ALL {
            assert_eq!(Model::from_product_id(model.product_id()), Some(model));
        }
        assert_eq!(Model::from_product_id(0x006E), Some(Model::Essential));
        assert_eq!(Model::from_product_id(0x0071), Some(Model::EssentialWhite));
        assert_eq!(Model::from_product_id(0x0098), Some(Model::Essential2021));
    }

    #[test]
    fn other_razer_mice_are_rejected() {
        // DeathAdder Elite, V2, V2 Mini, V3, Mamba, Viper... none may match.
        for pid in [
            0x005C, 0x0084, 0x008C, 0x00B2, 0x0044, 0x0078, 0x0000, 0xFFFF,
        ] {
            assert_eq!(Model::from_product_id(pid), None, "{pid:#06x}");
        }
    }

    #[test]
    fn the_2021_model_has_no_scroll_led() {
        assert_eq!(Model::Essential2021.leds(), &[Led::Logo]);
        assert_eq!(Model::Essential.leds(), &[Led::Logo, Led::Scroll]);
        assert_eq!(Model::EssentialWhite.leds(), &[Led::Logo, Led::Scroll]);
    }

    #[test]
    fn names_are_distinct() {
        let mut names: Vec<_> = Model::ALL.iter().map(|m| m.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 3);
    }
}
