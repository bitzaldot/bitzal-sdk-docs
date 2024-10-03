fn main() {
	#[cfg(feature = "std")]
	{
		matter_wasm_builder::WasmBuilder::new()
			.with_current_project()
			.export_heap_base()
			.import_memory()
			.build();
	}
}
