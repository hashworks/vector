/// A custom attribute on a container, variant, or field.
///
/// Applied by using the `#[configurable(metadata(...))]` helper. Two forms are supported:
///
/// - as a flag (`#[configurable(metadata(some_flag))]`)
/// - as a key/value pair (`#[configurable(metadata(status = "beta"))]`)
///
/// Custom attributes are added to the relevant schema definition as a custom field, `_metadata`, and stored as an
/// object. For key/value pairs, they are added as-is to the object. For flags, the flag name is the property name, and
/// the value will always be `true`.
#[derive(Clone, Debug)]
pub enum CustomAttribute {
    /// A standalone flag.
    ///
    /// Common for marking items as supporting a particular feature i.e. marking fields that can use the event template syntax.
    Flag(String),

    /// A key/value pair.
    ///
    /// Used for most metadata, where a given key could have many different possible values i.e. the status of a
    /// component (alpha, beta, stable, deprecated, etc).
    KeyValue { key: String, value: String },
}
