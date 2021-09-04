use git2::StatusEntry;

pub trait StatusEntryExt {
    fn is_conflicted(&self) -> bool;
    fn is_unstaged(&self) -> bool;
    fn is_staged(&self) -> bool;
}

impl StatusEntryExt for StatusEntry<'_> {
    fn is_conflicted(&self) -> bool {
        self.status().is_conflicted()
    }

    // ignores is_wt_new
    fn is_unstaged(&self) -> bool {
        self.status().is_wt_modified()
            || self.status().is_wt_deleted()
            || self.status().is_wt_typechange()
            || self.status().is_wt_renamed()
    }

    fn is_staged(&self) -> bool {
        self.status().is_index_new()
            || self.status().is_index_modified()
            || self.status().is_index_deleted()
            || self.status().is_index_typechange()
            || self.status().is_index_renamed()
    }
}
