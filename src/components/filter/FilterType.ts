type FilterTypePosibleVal = Record<string, { name: string; icon: string }>;

const FilterReadingStatus: FilterTypePosibleVal = {
	Liked: { name: "liked", icon: "heart" },
	Reading: { name: "reading", icon: "book-open" },
	Bookmarked: { name: "bookmarked", icon: "bookmark" },
	Finished: { name: "finished", icon: "check-circle" },
};

const FilterSortBy: FilterTypePosibleVal = {
	Alphabetical: { name: "alphabetical", icon: "font-case" },
	AddDate: { name: "add date", icon: "calendar-plus" },
	ReleaseDate: { name: "release date", icon: "calendar" },
	UpdateDate: { name: "update date", icon: "calendar-clock" },

	// LastRead: { name: "last read", icon: "calendar-check" },
};

const FilterSortOrder: FilterTypePosibleVal = {
	Ascending: { name: "ascending | newest", icon: "arrow-down-a-z" },
	Descending: { name: "descending | oldest", icon: "arrow-up-z-a" },
};

enum FilterType {
	ReadingStatus = "ReadingStatus",
	SortResult = "SortResult",
	SortOrder = "SortOrder",
}

export { FilterReadingStatus, FilterSortBy, FilterSortOrder, FilterType };
export type { FilterTypePosibleVal };
