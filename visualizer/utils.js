
export function is_number(str) {
    return str.match(/^[0-9]+$/)
}

export function capitalize(word) {
    if (!word.match(/^[a-zA-Z0-9.]+$/)) {
        throw new Error(`Asked to capitalize non-word ${word}`)
    }
    return word.charAt(0).toUpperCase() + word.slice(1)
}

export function remove_suffix(str, expected_suffix) {
    if (str.length < expected_suffix.length) {
        throw new Error(`Suffix '${expected_suffix}' not in string`)
    }
    const suffix = str.substring(str.length - expected_suffix.length)
    if (suffix != expected_suffix) {
        throw new Error(`Suffix '${expected_suffix}' not in string`)
    }
    return str.substring(0, str.length - expected_suffix.length)
}

export function remove_prefix(str, expected_prefix) {
    if (str.length < expected_prefix.length) {
        throw new Error(`Prefix '${expected_prefix}' not in string`)
    }
    const prefix = str.substring(0,expected_prefix.length)
    if (prefix != expected_prefix) {
        throw new Error(`Prefix '${expected_prefix}' not in string`)
    }
    return str.substring(expected_prefix.length)
}

/**
 * Parse the textual representation of an array of names
 *
 * E.g. "[a,b, c]" -> ["a","b","c"]
 *
 * @param {string} str The array string
 * @returns {string[]}
 */
export function ids_from_id_array_string(str) {
    let without_brackets = remove_prefix(str, '[')
    without_brackets = remove_suffix(without_brackets, ']')

    const result = []
    for (let id of without_brackets.split(",")) {
        id = id.trim()
        if (!id.match(/^[a-zA-Z0-9_]+$/)) {
            throw new Error(`Invalid id '${id}' in array`)
        }
        result.push(id)
    }

    return result
}

/**
 * Convert an ID from the JSON to a shorter, more user friendly string.
 *
 * E.g. "read_small_output_tmp_f51a9_17_limb_0" -> "ReadSmallOut17.0"
 *
 * @param {string} name
 * @param {string} containing_air_fn_name
 * @returns {string}
 */
export function shorten_name(name, containing_air_fn_name) {
    let i
    let result = name

    // Remove "_tmp_<hex>_"
    result = result.replace(/_tmp_[a-f0-9]{5}_/, "_")

    // Remove function name prefix
    if (result.startsWith(containing_air_fn_name + "_")) {
        result = remove_prefix(result, containing_air_fn_name + "_")
    }

    // Convert snake case to camel case
    const parts = result.split("_")
    const camel_case_parts = []
    for  (i = 0; i < parts.length; i++) {
        const part = parts[i]
        if (is_number(part)) {
            if (i > 0 && is_number(camel_case_parts[camel_case_parts.length - 1].slice(-1))) {
                camel_case_parts.push("_")
            }
            camel_case_parts.push(part)
        } else {
            let short_part
            switch (part) {
                case "input":
                    short_part = "in"
                    break
                case "output":
                    short_part = "out"
                    break
                case "limb":
                    if (i < parts.length - 1 && is_number(parts[i+1])) {
                        // Shorten "value_limb_3" to "value.3", but not "max_limb_size" to "max.size"
                        short_part = "."
                        break
                    }
                    // fallthrough
                default:
                    if (is_number(part.slice(-1))) {
                        // Don't truncate words that end with a number (e.g. "offset2")
                        short_part = part
                    } else {
                        short_part = part.slice(0,5)
                    }
                    break
            }
            camel_case_parts.push(capitalize(short_part))
        }
    }

    return camel_case_parts.join("")
}

export class DefaultMap {
    constructor(make_value) {
        this.make_value = make_value
        this.map = new Map()
    }

    has(key) {
        return this.map.has(key)
    }

    get(key) {
        if (!this.map.has(key)) {
            this.map.set(key, this.make_value())
        }
        return this.map.get(key)
    }

    set(key, value) {
        return this.map.set(key, value)
    }
}

/**
 * @param {any[]} array1
 * @param {any[]} array2
 * @returns {any[][]}
 */
export const zip = (array1, array2) => array1.map((v, i) => [v, array2[i]]);
