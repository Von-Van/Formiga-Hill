//! Hill's own souvenirs: the only rewards a story can give. A package names one of these by id
//! and Hill's catalogue says what it is; a package can never invent a reward, and nothing here
//! reaches Desktop unless Desktop one day lists it among what it accepts.

/// Every souvenir: its id and what it is called.
const CATALOGUE: [(&str, &str); 6] = [
    ("picnic_ribbon", "A gingham ribbon from the first picnic"),
    ("pressed_daisy", "A daisy pressed flat"),
    ("well_penny", "A penny from the bottom of the well"),
    ("oak_acorn", "An acorn from the old oak"),
    ("swing_feather", "A feather caught on the swing"),
    ("chest_marble", "A marble from the toy chest"),
];

pub fn exists(id: &str) -> bool {
    CATALOGUE.iter().any(|(known, _)| *known == id)
}

pub fn ids() -> Vec<&'static str> {
    CATALOGUE.iter().map(|(id, _)| *id).collect()
}

/// What a souvenir is called, for showing.
pub fn name(id: &str) -> Option<&'static str> {
    CATALOGUE
        .iter()
        .find(|(known, _)| *known == id)
        .map(|(_, name)| *name)
}
