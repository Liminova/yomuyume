export type JXLModule = EmscriptenWasm.Module & {
	decode(data: BufferSource): ImageData | null;
};

declare let moduleFactory: EmscriptenWasm.ModuleFactory<JXLModule>;

export default moduleFactory;
