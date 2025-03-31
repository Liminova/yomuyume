export type FilterTypePosibleVal = Record<string, { name: string; icon: string }>;

export const FilterReadingStatus: FilterTypePosibleVal = {
	Liked: { name: "liked", icon: "heart" },
	Reading: { name: "reading", icon: "book-open" },
	Bookmarked: { name: "bookmarked", icon: "bookmark" },
	Finished: { name: "finished", icon: "check-circle" },
};

export const FilterSortBy: FilterTypePosibleVal = {
	Alphabetical: { name: "alphabetical", icon: "font-case" },
	AddDate: { name: "add date", icon: "calendar-plus" },
	ReleaseDate: { name: "release date", icon: "calendar" },
	UpdateDate: { name: "update date", icon: "calendar-clock" },

	// LastRead: { name: "last read", icon: "calendar-check" },
};

export const FilterSortOrder: FilterTypePosibleVal = {
	Ascending: { name: "ascending | newest", icon: "arrow-down-a-z" },
	Descending: { name: "descending | oldest", icon: "arrow-up-z-a" },
};

/* eslint-disable no-unused-vars */
export enum FilterType {
	ReadingStatus = "reading-status",
	SortResult = "sort-result",
	SortOrder = "sort-order",
}
