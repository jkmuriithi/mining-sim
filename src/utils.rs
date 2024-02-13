//! Helper functions and data types

/// Uses a name string to turn any pure `Fn(Input) -> Output` into a sized,
/// sortable, hashable, clone-able, and thread-safe datatype.
#[derive(Clone)]
pub struct WrapFunc<Input, Output> {
    name: String,
    func: std::sync::Arc<dyn Fn(Input) -> Output + Send + Sync + 'static>,
}

/// Creates a `WrapFunc` from a name and a closure/function.
macro_rules! wrap {
    ($name:expr, $func:expr) => {
        WrapFunc::new($name, $func)
    };
}

pub(crate) use wrap;

impl<I, J> WrapFunc<I, J> {
    pub fn new<N, F>(name: N, func: F) -> Self
    where
        N: Into<String>,
        F: Fn(I) -> J + Send + Sync + 'static,
    {
        Self { name: name.into(), func: std::sync::Arc::new(func) }
    }

    pub fn call(&self, input: I) -> J {
        self.func.as_ref()(input)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<I, J> std::fmt::Debug for WrapFunc<I, J> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<WrappedFunc: {}>", self.name)
    }
}

impl<I, J> PartialEq for WrapFunc<I, J> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl<I, J> Eq for WrapFunc<I, J> {}

impl<I, J> PartialOrd for WrapFunc<I, J> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl<I, J> Ord for WrapFunc<I, J> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<I, J> std::hash::Hash for WrapFunc<I, J> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[inline]
pub fn median_of_floats(mut values: Vec<f64>) -> f64 {
    debug_assert!(!values.is_empty(), "median of empty vec");

    values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let len = values.len();
    let mid = len >> 1;

    if len & 1 == 0 {
        (values[mid - 1] + values[mid]) * 0.5
    } else {
        values[mid]
    }
}
