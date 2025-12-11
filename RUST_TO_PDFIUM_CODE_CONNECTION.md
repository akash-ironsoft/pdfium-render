# How Rust Code Connects to pdfium.js - Code Level Explanation

This document shows the **actual code** that ties Rust WASM to pdfium.js/pdfium.wasm.

## Step-by-Step Code Flow

### Step 1: Browser Loads Both Modules

**File: `release/index.html:315-334`**

```javascript
// Load Pdfium C++ module first
PDFiumModule().then(async pdfiumModule => {
    console.log('✓ Pdfium WASM module loaded (custom build with QPDF)');

    // Store it globally so Rust can access it
    window.PDFiumModule = pdfiumModule;

    // Load Rust module
    const rustModule = await init('pdfium_render_wasm_example_bg.wasm');
    console.log('✓ Rust WASM module loaded');

    // CRITICAL STEP: Connect the two modules
    const initialized = initialize_pdfium_render(
        pdfiumModule,    // ← Pass Pdfium to Rust
        rustModule,      // ← Pass Rust module
        false            // ← Debug mode off
    );

    if (!initialized) {
        console.error('Initialization failed');
        return;
    }

    console.log('✓ pdfium-render initialized successfully');
});
```

**What happens**:
1. `PDFiumModule()` loads pdfium.wasm and returns JavaScript object with functions
2. `init()` loads Rust WASM and returns JavaScript object with functions
3. `initialize_pdfium_render()` **connects them together** ← This is the key!

---

### Step 2: Rust Stores Reference to Pdfium Module

**File: `src/bindings/wasm_bindings.rs:1028-1070`**

```rust
#[wasm_bindgen]  // ← Makes this callable from JavaScript
pub fn initialize_pdfium_render(
    pdfium_wasm_module: JsValue,   // ← The pdfium.js object
    local_wasm_module: JsValue,    // ← The Rust module object
    debug: bool,
) -> bool {
    // Set up logging
    console_log::init_with_level(if debug {
        log::Level::Trace
    } else {
        log::Level::Info
    }).ok();

    if debug {
        console_error_panic_hook::set_once();
    }

    if pdfium_wasm_module.is_object() && local_wasm_module.is_object() {
        // Store the Pdfium module in global state
        match PdfiumRenderWasmState::lock_mut().bind_to_pdfium(
            Object::from(pdfium_wasm_module),   // ← Stored here!
            Object::from(local_wasm_module),
            debug,
        ) {
            Ok(()) => true,
            Err(msg) => {
                log::error!("initialize_pdfium_render(): {}", msg);
                false
            }
        }
    } else {
        log::error!("Provided modules are not valid Javascript Objects");
        false
    }
}
```

**What happens**: The `pdfium_wasm_module` object is stored in a global singleton called `PdfiumRenderWasmState`.

---

### Step 3: Storing Pdfium Functions

**File: `src/bindings/wasm_bindings.rs:132-196`**

```rust
impl PdfiumRenderWasmState {
    /// Stores references to Pdfium functions
    fn bind_to_pdfium(
        &mut self,
        pdfium_wasm_module: Object,
        local_wasm_module: Object,
        debug: bool,
    ) -> Result<(), &str> {
        // Store the module object
        self.pdfium_wasm_module = Some(pdfium_wasm_module);
        self.local_wasm_module = Some(local_wasm_module);
        self.debug = debug;

        // Extract malloc function from Pdfium module
        self.malloc_js_fn = Some(Function::from(
            self.get_value_from_pdfium_wasm_module("_malloc")
                .or_else(|_| {
                    // Fallback: try Module['wasmExports'].malloc
                    self.get_value_from_pdfium_wasm_module("wasmExports")
                        .and_then(|exports| {
                            Reflect::get(&exports, &JsValue::from("malloc"))
                                .map_err(|_| "malloc not found in wasmExports")
                        })
                })?,
        ));

        // Extract free function
        self.free_js_fn = Some(Function::from(
            self.get_value_from_pdfium_wasm_module("_free")
                .or_else(|_| {
                    // Fallback: try Module['wasmExports'].free
                    self.get_value_from_pdfium_wasm_module("wasmExports")
                        .and_then(|exports| {
                            Reflect::get(&exports, &JsValue::from("free"))
                                .map_err(|_| "free not found in wasmExports")
                        })
                })?,
        ));

        // Extract ccall function (Emscripten helper)
        self.call_js_fn = Some(Function::from(
            self.get_value_from_pdfium_wasm_module("ccall")
                .map_err(|_| "Module.ccall() not defined")?,
        ));

        // Verify HEAPU8 array exists
        if self.get_value_from_pdfium_wasm_module("HEAPU8").is_err() {
            return Err("Module.HEAPU8[] not defined");
        }

        Ok(())
    }

    /// Gets a value from the Pdfium WASM module
    fn get_value_from_pdfium_wasm_module(&self, key: &str) -> Result<JsValue, &str> {
        Reflect::get(
            self.pdfium_wasm_module.as_ref().unwrap(),
            &JsValue::from(key),
        )
        .map_err(|_| "Property not found")
    }
}
```

**What's stored**:
- `pdfium_wasm_module`: The entire Pdfium module object
- `malloc_js_fn`: Function to allocate memory in Pdfium's heap
- `free_js_fn`: Function to free memory in Pdfium's heap
- `call_js_fn`: The `ccall()` function from Emscripten

---

### Step 4: Global State Storage

**File: `src/bindings/wasm_bindings.rs:55-85`**

```rust
// Global singleton storing Pdfium bindings
static PDFIUM_RENDER_WASM_STATE: Lazy<RwLock<PdfiumRenderWasmState>> =
    Lazy::new(|| RwLock::new(PdfiumRenderWasmState::default()));

/// State containing references to Pdfium WASM module
#[derive(Debug)]
pub(crate) struct PdfiumRenderWasmState {
    pdfium_wasm_module: Option<Object>,     // ← The pdfium.js object
    local_wasm_module: Option<Object>,      // ← The Rust module object
    wasm_table: Option<WebAssembly::Table>, // ← Function table
    malloc_js_fn: Option<Function>,         // ← malloc function
    free_js_fn: Option<Function>,           // ← free function
    call_js_fn: Option<Function>,           // ← ccall function
    debug: bool,
    file_access_callback_function_table_entry: usize,
    file_write_callback_function_table_entry: usize,
    state: HashMap<String, JsValue>,        // ← Extra state storage
}

impl PdfiumRenderWasmState {
    /// Get read-only access to global state
    pub fn lock() -> RwLockReadGuard<'static, PdfiumRenderWasmState> {
        PDFIUM_RENDER_WASM_STATE.try_read().unwrap()
    }

    /// Get read-write access to global state
    pub fn lock_mut() -> RwLockWriteGuard<'static, PdfiumRenderWasmState> {
        PDFIUM_RENDER_WASM_STATE.try_write().unwrap()
    }
}
```

**Key Point**: This is a **global singleton** accessible from anywhere in the Rust code. Any Rust function can access the Pdfium module by calling `PdfiumRenderWasmState::lock()`.

---

### Step 5: Calling Pdfium Functions from Rust

**File: `src/bindings/wasm_bindings.rs:358-450`**

```rust
impl PdfiumRenderWasmState {
    /// Call a Pdfium function using Emscripten's ccall
    fn call(
        &self,
        fn_name: &str,                                  // ← Function name (e.g., "FPDF_InitLibrary")
        return_type: JsFunctionArgumentType,            // ← Return type
        arg_types: Option<Vec<JsFunctionArgumentType>>, // ← Argument types
        args: Option<&JsValue>,                         // ← Actual arguments
    ) -> JsValue {
        log::debug!("Calling function: {}", fn_name);

        let js_fn_name = JsValue::from(intern(fn_name));
        let js_return_type = js_value_from_argument_type(return_type);

        let js_arg_types = match arg_types {
            Some(types) => JsValue::from(
                types.into_iter()
                    .map(|t| js_value_from_argument_type(t))
                    .collect::<Array>()
            ),
            None => JsValue::undefined(),
        };

        // Call Module.ccall(fn_name, return_type, arg_types, args)
        match self.call_js_fn.as_ref().unwrap().apply(
            &JsValue::null(),
            &Array::of4(
                &js_fn_name,      // ← Function name
                &js_return_type,  // ← Return type ("number", "string", etc.)
                &js_arg_types,    // ← Array of arg types
                &args.unwrap_or(&JsValue::undefined()),  // ← Arguments
            ),
        ) {
            Ok(result) => {
                log::debug!("Function {} returned: {:?}", fn_name, result);
                result
            }
            Err(err) => {
                log::error!("Error calling {}: {:?}", fn_name, err);
                JsValue::undefined()
            }
        }
    }
}
```

**What this does**:
```javascript
// Rust calls state.call("FPDF_InitLibrary", ...)
// Which translates to JavaScript:
Module.ccall(
    "FPDF_InitLibrary",  // Function name
    null,                // Return type (void)
    [],                  // No arguments
    []                   // No argument values
);
```

---

### Step 6: Example - Rust Function Calling Pdfium

**File: `src/bindings/wasm_bindings.rs:1595-1612`**

```rust
impl PdfiumLibraryBindings for WasmPdfiumBindings {
    #[allow(non_snake_case)]
    fn FPDF_InitLibrary(&self) {
        log::debug!("Calling FPDF_InitLibrary()");

        let state = PdfiumRenderWasmState::lock();  // ← Get global state

        // Check which init function exists
        let init = if state
            .get_value_from_pdfium_wasm_module("FPDF_InitLibrary")
            .is_ok()
        {
            "FPDF_InitLibrary"
        } else {
            "PDFium_Init"  // Fallback name
        };

        // Call the function through ccall
        state.call(
            init,                              // ← Function name
            JsFunctionArgumentType::Void,      // ← Returns void
            None,                              // ← No argument types
            None,                              // ← No arguments
        );
    }
}
```

**JavaScript equivalent**:
```javascript
Module.ccall("FPDF_InitLibrary", null, [], []);
```

---

### Step 7: Example with Arguments - Loading a Document

**File: `src/bindings/wasm_bindings.rs:1667-1708`**

```rust
impl PdfiumLibraryBindings for WasmPdfiumBindings {
    #[allow(non_snake_case)]
    fn FPDF_LoadMemDocument64(&self, data_buf: &[u8], password: Option<&str>) -> FPDF_DOCUMENT {
        log::debug!("FPDF_LoadMemDocument64(): entering");

        let mut state = PdfiumRenderWasmState::lock_mut();

        // Step 1: Copy PDF bytes to Pdfium's memory heap
        let ptr = state.copy_bytes_to_pdfium(data_buf);

        log::debug!("Copied {} bytes to Pdfium heap at address {}", data_buf.len(), ptr);

        // Step 2: Prepare password (if provided)
        let password_ptr = if let Some(password) = password {
            state.copy_string_to_pdfium(CString::new(password).unwrap().as_ptr())
        } else {
            0  // NULL pointer
        };

        // Step 3: Call FPDF_LoadMemDocument64
        let result = state
            .call(
                "FPDF_LoadMemDocument64",
                JsFunctionArgumentType::Pointer,  // ← Returns pointer
                Some(vec![
                    JsFunctionArgumentType::Pointer,  // ← data_buf pointer
                    JsFunctionArgumentType::Number,   // ← size
                    JsFunctionArgumentType::Pointer,  // ← password pointer
                ]),
                Some(&JsValue::from(Array::of3(
                    &Self::js_value_from_offset(ptr),                    // ← PDF data pointer
                    &JsValue::from(data_buf.len() as f64),              // ← PDF size
                    &Self::js_value_from_offset(password_ptr),          // ← Password pointer
                ))),
            )
            .as_f64()
            .unwrap() as usize as FPDF_DOCUMENT;

        // Step 4: Free the temporary buffers
        state.free(ptr);
        if password_ptr != 0 {
            state.free(password_ptr);
        }

        log::debug!("FPDF_LoadMemDocument64(): document handle = {}", result as usize);

        result
    }
}
```

**JavaScript equivalent**:
```javascript
// Allocate memory in Pdfium's heap
const pdfPtr = Module._malloc(pdfBytes.length);

// Copy PDF bytes into Pdfium's heap
Module.HEAPU8.set(pdfBytes, pdfPtr);

// Call Pdfium function
const docHandle = Module.ccall(
    "FPDF_LoadMemDocument64",
    "number",                          // Returns pointer (as number)
    ["number", "number", "number"],    // Arg types: ptr, size, password
    [pdfPtr, pdfBytes.length, 0]       // Actual arguments
);

// Free the temporary buffer
Module._free(pdfPtr);
```

---

### Step 8: Memory Management - Copying Between Heaps

**File: `src/bindings/wasm_bindings.rs:549-561`**

```rust
impl PdfiumRenderWasmState {
    /// Copy bytes from Rust heap to Pdfium heap
    fn copy_bytes_to_pdfium(&self, bytes: &[u8]) -> usize {
        log::debug!("Copying {} bytes to Pdfium", bytes.len());

        // Allocate memory in Pdfium's heap
        let remote_ptr = self.malloc(bytes.len());

        log::debug!("Allocated at Pdfium address: {}", remote_ptr);

        // Copy bytes into Pdfium's heap
        self.copy_bytes_to_pdfium_address(bytes, remote_ptr);

        remote_ptr
    }

    /// Copy bytes to specific address in Pdfium heap
    fn copy_bytes_to_pdfium_address(&self, bytes: &[u8], remote_ptr: usize) {
        // Access Pdfium's HEAPU8 array and write bytes
        self.heap_u8()
            .set(unsafe { &Uint8Array::view(bytes) }, remote_ptr as u32);

        log::debug!("Copied {} bytes into WASM heap at {}", bytes.len(), remote_ptr);
    }

    /// Call malloc in Pdfium's heap
    fn malloc(&self, len: usize) -> usize {
        self.malloc_js_fn
            .as_ref()
            .unwrap()
            .call1(&JsValue::null(), &JsValue::from(len as f64))
            .unwrap()
            .as_f64()
            .unwrap() as usize
    }

    /// Call free in Pdfium's heap
    fn free(&self, ptr: usize) {
        self.free_js_fn
            .as_ref()
            .unwrap()
            .call1(&JsValue::null(), &JsValue::from(ptr as f64))
            .ok();
    }

    /// Access Pdfium's HEAPU8 byte array
    fn heap_u8(&self) -> Uint8Array {
        self.get_value_from_pdfium_wasm_module("HEAPU8")
            .unwrap()
            .dyn_into::<Uint8Array>()
            .unwrap()
    }
}
```

**Why this is needed**:
- Rust WASM and Pdfium WASM have **separate memory heaps**
- They cannot directly share pointers
- Data must be **copied** between heaps

**JavaScript equivalent**:
```javascript
// Allocate in Pdfium heap
const pdfiumPtr = Module._malloc(1024);

// Copy from Rust heap to Pdfium heap
Module.HEAPU8.set(rustBytes, pdfiumPtr);

// Use the pointer in Pdfium functions
const result = Module.ccall("FPDF_SomeFunction", "number", ["number"], [pdfiumPtr]);

// Free when done
Module._free(pdfiumPtr);
```

---

## Complete Call Stack Example: pdf_to_qpdf_json()

Let's trace the complete flow with actual code:

### 1. User clicks button in browser

```javascript
// index.html:292
async function convertToQpdfJson() {
    const json = await pdfToQpdfJson("test.pdf", 2);
    console.log(json);
}
```

### 2. JavaScript calls Rust WASM function

```javascript
// pdfium_render_wasm_example.js (generated by wasm-bindgen)
export function pdf_to_qpdf_json(url, version) {
    // Convert JS string to Rust string
    const urlPtr = passStringToWasm(url);

    // Call Rust WASM
    const resultPtr = wasm.pdf_to_qpdf_json(urlPtr, version);

    // Convert Rust string to JS string
    return getStringFromWasm(resultPtr);
}
```

### 3. Rust WASM function executes

```rust
// examples/wasm.rs:198
#[wasm_bindgen]
pub async fn pdf_to_qpdf_json(url: String, version: i32) -> Result<String, JsValue> {
    let pdfium = get_pdfium();

    // Load PDF
    let document = pdfium
        .load_pdf_from_fetch(url, None)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {:?}", e)))?;

    // Convert to JSON
    document
        .to_qpdf_json(version)
        .map_err(|e| JsValue::from_str(&format!("QPDF conversion failed: {:?}", e)))
}
```

### 4. Rust calls internal API

```rust
// src/pdf/document.rs:418
pub fn to_qpdf_json(&self, version: i32) -> Result<String, PdfiumError> {
    let pdf_bytes = self.to_bytes()?;

    // Call FFI binding
    self.bindings().IPDF_QPDF_PDFToJSON(&pdf_bytes, version)
}
```

### 5. FFI binding calls Pdfium WASM

```rust
// src/bindings/wasm_bindings.rs (hypothetical implementation)
fn IPDF_QPDF_PDFToJSON(&self, pdf_bytes: &[u8], version: i32) -> Result<String, PdfiumError> {
    let state = PdfiumRenderWasmState::lock();

    // Copy PDF to Pdfium heap
    let pdf_ptr = state.copy_bytes_to_pdfium(pdf_bytes);

    // Call IPDF_QPDF_PDFToJSON in pdfium.wasm
    let json_ptr = state.call(
        "IPDF_QPDF_PDFToJSON",
        JsFunctionArgumentType::Pointer,
        Some(vec![
            JsFunctionArgumentType::Pointer,  // pdf_data
            JsFunctionArgumentType::Number,   // size
            JsFunctionArgumentType::Number,   // version
        ]),
        Some(&JsValue::from(Array::of3(
            &Self::js_value_from_offset(pdf_ptr),
            &JsValue::from(pdf_bytes.len() as f64),
            &JsValue::from(version as f64),
        ))),
    ).as_f64().unwrap() as usize;

    // Copy result back from Pdfium heap
    let json_string = state.copy_string_from_pdfium(json_ptr);

    // Free memory
    state.free(pdf_ptr);

    Ok(json_string)
}
```

### 6. JavaScript ccall executes

```javascript
// Inside pdfium.js (Emscripten generated)
Module.ccall = function(ident, returnType, argTypes, args) {
    // Look up function in exported table
    const func = Module['_' + ident];

    // Call the function in pdfium.wasm
    const result = func(args[0], args[1], args[2]);

    return result;
};
```

### 7. Pdfium WASM executes C++ code

```cpp
// fpdfsdk/fpdf_qpdf.cpp:32
const char* IPDF_QPDF_PDFToJSON(const unsigned char* pdf_data, int size, int version) {
    try {
        QPDF qpdf;
        qpdf.processMemoryFile(NULL, (const char*)pdf_data, size);

        std::stringstream output;
        qpdf.writeJSON(version, output, ...);

        std::string json = output.str();
        return strdup(json.c_str());  // ← Allocated in Pdfium heap
    } catch (std::exception& e) {
        // Return error JSON
        return strdup("{\"error\":true,\"message\":\"...\"}");
    }
}
```

### 8. Result bubbles back up

```
Pdfium C++ → JavaScript (ccall result) → Rust (copy from Pdfium heap)
→ Rust WASM → JavaScript (wasm-bindgen) → Browser console
```

---

## Visual Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                         BROWSER                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  JavaScript Code (index.html)                                    │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ await pdfToQpdfJson("test.pdf", 2)                         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ pdfium_render_wasm_example.js (wasm-bindgen glue)         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Rust WASM Module (pdfium_render_wasm_example_bg.wasm)    │ │
│  │ ┌────────────────────────────────────────────────────────┐ │ │
│  │ │ fn pdf_to_qpdf_json() {                                │ │ │
│  │ │   // Access global state                               │ │ │
│  │ │   let state = PdfiumRenderWasmState::lock();         │ │ │
│  │ │                                                         │ │ │
│  │ │   // Copy data to Pdfium heap                         │ │ │
│  │ │   let ptr = state.copy_bytes_to_pdfium(pdf_bytes);   │ │ │
│  │ │   //          ↓ calls state.malloc_js_fn (Module._malloc)│ │
│  │ │   //          ↓ writes to state.heap_u8 (Module.HEAPU8) │ │
│  │ │                                                         │ │ │
│  │ │   // Call Pdfium function                             │ │ │
│  │ │   let result = state.call(                            │ │ │
│  │ │       "IPDF_QPDF_PDFToJSON",                         │ │ │
│  │ │       return_type, args                               │ │ │
│  │ │   );                                                   │ │ │
│  │ │   //          ↓ calls state.call_js_fn (Module.ccall) │ │ │
│  │ │ }                                                       │ │ │
│  │ └────────────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ pdfium.js (Emscripten glue)                               │ │
│  │ ┌────────────────────────────────────────────────────────┐ │ │
│  │ │ Module.ccall("IPDF_QPDF_PDFToJSON", ...)             │ │ │
│  │ │   // Look up function in exports                       │ │ │
│  │ │   const func = Module._IPDF_QPDF_PDFToJSON;          │ │ │
│  │ │   return func(ptr, size, version);                     │ │ │
│  │ └────────────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          ↓                                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ pdfium.wasm (Compiled C++)                                │ │
│  │ ┌────────────────────────────────────────────────────────┐ │ │
│  │ │ const char* IPDF_QPDF_PDFToJSON(...) {               │ │ │
│  │ │   QPDF qpdf;                                           │ │ │
│  │ │   qpdf.processMemoryFile(...);                         │ │ │
│  │ │   qpdf.writeJSON(...);                                 │ │ │
│  │ │   return json_ptr;                                     │ │ │
│  │ │ }                                                       │ │ │
│  │ └────────────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘

Key Connections:
═══════════════
1. initialize_pdfium_render(pdfiumModule, rustModule, debug)
   → Stores pdfiumModule in PdfiumRenderWasmState global singleton

2. PdfiumRenderWasmState stores:
   - pdfium_wasm_module: The pdfium.js Module object
   - malloc_js_fn: Module._malloc
   - free_js_fn: Module._free
   - call_js_fn: Module.ccall
   - heap_u8: Module.HEAPU8

3. Rust calls state.call(fn_name, return_type, arg_types, args)
   → JavaScript: call_js_fn.apply(...)
   → JavaScript: Module.ccall(fn_name, ...)
   → pdfium.wasm: Executes C++ function
```

---

## Summary

**Q: How does Rust tie to pdfium.js?**

**A: Through a global singleton that stores function references:**

1. **Initialization**: `initialize_pdfium_render(pdfiumModule, rustModule)` stores the pdfium.js `Module` object in `PdfiumRenderWasmState` singleton

2. **Function Storage**: Extracts and stores key functions:
   - `Module._malloc` → `malloc_js_fn`
   - `Module._free` → `free_js_fn`
   - `Module.ccall` → `call_js_fn`
   - `Module.HEAPU8` → Direct access to Pdfium's memory

3. **Function Calls**: Any Rust code can:
   ```rust
   let state = PdfiumRenderWasmState::lock();
   state.call("FPDF_SomeFunction", return_type, args);
   ```
   Which translates to JavaScript:
   ```javascript
   Module.ccall("FPDF_SomeFunction", ...);
   ```

4. **Memory Management**: Data is copied between heaps:
   ```rust
   // Rust → Pdfium
   let ptr = state.malloc(size);           // Module._malloc(size)
   state.heap_u8().set(bytes, ptr);        // Module.HEAPU8.set(bytes, ptr)

   // Call function with pointer
   state.call("FPDF_LoadMemDocument", ..., [ptr, size]);

   // Pdfium → Rust
   let result = state.heap_u8().slice(ptr, len);  // Module.HEAPU8.slice(...)
   state.free(ptr);                        // Module._free(ptr)
   ```

**The key insight**: Rust doesn't directly call C. Instead, it calls JavaScript functions (`ccall`, `malloc`, `free`) that were extracted from the pdfium.js Module object during initialization. These JavaScript functions then invoke the actual C++ code running in pdfium.wasm.
