
import htm from "./htm/index.js"

function flat_append(elem, nested_children) {
    for (const child of nested_children) {
        if (Array.isArray(child)) {
            flat_append(elem, child)
        } else {
            elem.append(child)
        }
    }
}

function create_element(tag, props, ...children) {
    // Disable caching (see htm/README.md)
    // With caching, multiple calls to html`...<element>Some content</element>...` can reuse
    // <element>. This is problematic because this function returns a HTMLElement, and
    // adding the same HTMLElement to multiple parents is forbidden.
    this[0] = 3;

    let result
    if (tag == "svg" || tag == "rect" || tag == "circle" || tag == "polygon" || tag == "path") {
        result = document.createElementNS("http://www.w3.org/2000/svg", tag)
    } else {
        result = document.createElement(tag)
    }

    for (const key in props) {
        result.setAttribute(key, props[key])
    }

    flat_append(result, children)
    return result
}

/**
 * @param {any[]} array
 * @param {any} separator
 * @returns {any[]}
 */
export function intersperse(array, separator) {
    return array.flatMap((x, i) => i == 0 ? [x]: [separator, x])
}

export const html = htm.bind(create_element)

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
 * Are every one of these vars individually single-use? Used to gray out a collapsed group's
 * label when all the vars it summarizes would themselves be grayed out.
 * @param {import("./air.js").Var[]} vars
 */
export function all_used_once(vars, air_view) {
    return vars.every((v) => air_view.air.usage_counts.get(v.id) === 1)
}

export function create_var_span(var_obj, air_view) {
    const id = var_obj.id
    const result = html`
        <span title=${var_obj.id}>
            ${var_obj.display_name}
        </span>`

    if (air_view.air.usage_counts.get(id) === 1) {
        result.classList.add('used-once')
    }

    result.addEventListener('click', (e) => air_view.select_vars([id]))
    air_view.var_select_listeners.push((selected_ids) => {
        result.classList.remove('highlighted-full')
        result.classList.remove('highlighted-half')
        result.classList.remove('highlighted-none')
        if (selected_ids.includes(id)) {
            result.classList.add('highlighted-full')
        } else {
            result.classList.add('highlighted-none')
        }
    })

    // Don't allow the var to be selected with a double-click. We use double-click for
    // template expand / collapse
    result.addEventListener('mousedown', (e) => {
        if (e.detail > 1) {
            e.preventDefault()
        }
    })
    return result
}

/**
 * @param {any[]} array1
 * @param {any[]} array2
 * @returns {any[][]}
 */
export const zip = (array1, array2) => array1.map((v, i) => [v, array2[i]]);
