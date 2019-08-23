local ffi = require('ffi')

local ext

if ffi.os == 'Linux' then
    ext = 'so'
else
    ext = 'dylib'
end

ffi.cdef [[
const char* ffi_select(const char *json_str, const char *path);
void *ffi_path_compile(const char *path);
const char* ffi_select_with_compiled_path(void *ptr, const char *json_str);
]]

local jsonpathLibPath = os.getenv("JSONPATH_LIB_PATH");
local jsonpath = ffi.load(jsonpathLibPath .. '/libjsonpath_lib.' .. ext);

function compile(path)
    local compiledPath = jsonpath.ffi_path_compile(path);
    return function(jsonStr)
        return ffi.string(jsonpath.ffi_select_with_compiled_path(compiledPath, jsonStr));
    end
end