const path = require("path")
const os = require("os")

const EXTENSION = (() =>{
    if (os.platform() === "win32") {
        return `.exe`
    } else {
        return ""
    }
})()

const SCRIPT_GEN_PATH = path.join(__dirname, "bin")

const SCRIPT_GEN_NAME = `kic-script-gen${EXTENSION}`
const SCRIPT_GEN_EXECUTABLE = path.join(SCRIPT_GEN_PATH, SCRIPT_GEN_NAME)

module.exports = {
    SCRIPT_GEN_NAME,
    SCRIPT_GEN_PATH,
    SCRIPT_GEN_EXECUTABLE,
}
