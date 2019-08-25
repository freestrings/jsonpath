require("init")
local jsonpath = require("jsonpath")

local args = ngx.req.get_uri_args()

local path = args['path']
if(path == nil or path == '') then
    return ngx.exit(400)
end

local template = jsonpath.compile(path)
local data = ngx.location.capture("/example.json")
local result = template(data.body)
ngx.say(result)