const DEFAULT_LIMIT: usize = 100;


#[derive(Clone)]
pub struct QueryOptions {
    pub(crate) limit: usize,
    pub(crate) ascending: bool,
    pub(crate) cutoff: f64,
}

impl QueryOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.limit = limit;
        self
    }

    pub fn ascending(&mut self, ascending: bool) -> &mut Self {
        self.ascending = ascending;
        self
    }

    /// Cutoff value defines the point where values are cut off.
    /// For example if ascending is true, e.g. we have a sorted list like [1,2,3,4,5], with a
    /// cutoff of 4, the resulting vector is [1,2,3,4].
    /// If the sorting order is descending the list would be [5, 4].
    /// This is important, because of the different comparison operator used
    pub fn cutoff(&mut self, cutoff: f64) -> &mut Self {
        self.cutoff = cutoff;
        self
    }

    pub fn build(&mut self) -> Self {
        self.clone()
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            ascending: false,
            cutoff: -1.0,
        }
    }
}

#[derive(Clone)]
pub struct InsertOptions {
    pub(crate) limit: usize,
}

impl InsertOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.limit = limit;
        self
    }

    pub fn build(&mut self) -> Self {
        self.clone()
    }
}

impl Default for InsertOptions {
    fn default() -> Self {
        Self { limit: DEFAULT_LIMIT }
    }
}
