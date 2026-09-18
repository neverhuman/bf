use ts_rs::TS;
pub fn typescript() -> String {
    crate::commands::typescript()
}
pub(crate) fn declaration<T: TS>() -> String {
    format!("export {};\n", T::decl())
}
#[cfg(test)]
mod tests {
    #[test]
    fn shared_types_are_current() {
        assert_eq!(super::typescript(), include_str!("../web/src/wire.ts"));
    }
}
