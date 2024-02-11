//! Helper functions and data types

/// Helper for storing idempotent closures/functions with one argument
#[derive(Clone)]
pub struct WrappedFunc<Input, Output> {
    name: String,
    func: std::sync::Arc<dyn Fn(Input) -> Output + Send + Sync + 'static>,
}

impl<I, J> WrappedFunc<I, J> {
    pub fn new<N, F>(name: N, func: F) -> Self
    where
        N: Into<String>,
        F: Fn(I) -> J + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            func: std::sync::Arc::new(func),
        }
    }

    pub fn call(&self, input: I) -> J {
        self.func.as_ref()(input)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<I, J> std::fmt::Debug for WrappedFunc<I, J> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<WrappedFunc: {}>", self.name)
    }
}

impl<I, J> PartialEq for WrappedFunc<I, J> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl<I, J> Eq for WrappedFunc<I, J> {}

impl<I, J> PartialOrd for WrappedFunc<I, J> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl<I, J> Ord for WrappedFunc<I, J> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<I, J> std::hash::Hash for WrappedFunc<I, J> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
