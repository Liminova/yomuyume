type BreakpointRecord = Record<
	number,
	{
		slidesPerView: number;
		spaceBetween?: number;
	}
>;
type MyImage = {
	src: string;
	width?: number;
	height?: number;
	blurhash?: string;
	format: string;
};

type GenericSrvResponse = {
	message: string;
	ok: boolean;
};

export type { BreakpointRecord, MyImage, GenericSrvResponse };
