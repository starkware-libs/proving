/** @typedef {{type: "literal", value: string} | {type: "range", min: number, max: number}} TemplatePart */

import { html } from "./utils.js"

// Represents a sequence of strings, each built from letters and numbers. The letters
// are the same in all strings. Each number is either the same in all strings or advances
// by 1 in each string.
export class Template {
    /**
     * @param {number} count
     * @param {TemplatePart[]} parts
     */
    constructor(count, parts) {
        this.count = count
        this.parts = parts
    }

    /**
     * @param {string} str
     */
    add_string(str) {
        let i

        const tokens = tokenize_string(str)
        if (tokens.length != this.parts.length) {
            return false
        }

        for (i = 0; i < this.parts.length; i++) {
            const part = this.parts[i]
            const token = tokens[i]
            switch (part.type) {
                case "literal":
                    if (token.value + "" !== part.value) {
                        return false
                    }
                    break;
                case "range":
                    if (token.type != "digit") {
                        return false
                    }
                    if (token.value != part.max + 1) {
                        return false
                    }
                    part.max += 1
                    break;
            }
        }
        this.count += 1
        return true
    }

    get_html(air_view, var_ids) {
        let result = document.createElement("span")
        for (const part of this.parts) {
            switch (part.type) {
                case "literal":
                    result.append(part.value)
                    break
                case "range":
                    result.append(html`<span class="template-range">${part.min}…${part.max}</span>`)
                    break
            }
        }
        // Highlight the collapsed representation when it contains selected variables
        air_view.var_select_listeners.push((selected_ids) => {
            let selected_from_self = 0
            for (const id of var_ids) {
                if (selected_ids.includes(id)) {
                    selected_from_self++
                }
            }
            result.classList.remove('highlighted-full')
            result.classList.remove('highlighted-half')
            result.classList.remove('highlighted-none')
            if (selected_from_self == var_ids.length) {
                result.classList.add('highlighted-full')
            } else if (selected_from_self > 0) {
                result.classList.add('highlighted-half')
            } else {
                result.classList.add('highlighted-none')
            }
        })
        return result
    }

    /**
     * @returns {string}
     */
    get_text() {
        let result = ""
        for (const part of this.parts) {
            switch (part.type) {
                case "literal":
                    result += part.value
                    break
                case "range":
                    result += `${part.min}-${part.max}`
                    break
            }
        }
        return result
    }

    clone() {
        return new Template(this.count, structuredClone(this.parts))
    }
}

function char_type(char) {
    if (char >= "0" && char <= "9") {
        return "digit"
    }
    return "letter"
}

function make_token(type, value_str) {
    if (type == "digit") {
        return {type, value: parseInt(value_str)}
    }
    if (type == "letter") {
        return {type, value: value_str}
    }
    throw new Error(`Invalid token type '${type}`)
}

function tokenize_string(s) {
    const result = []
    let cur_token = ""
    let cur_token_type = ""
    for (const ch of s) {
        let ch_type = char_type(ch)
        if (cur_token_type != ch_type && cur_token_type != "") {
            result.push(make_token(cur_token_type, cur_token))
            cur_token = ""
        }
        cur_token_type = ch_type
        cur_token += ch
    }
    if (cur_token != "") {
        result.push(make_token(cur_token_type, cur_token))
    }
    return result
}

/**
 * @param {string[]} strings
 * @returns {Template}
 */
export function strings_maximal_template(strings) {
    let i

    /** @type {Template} */
    const single_string_template = new Template(1, [{type: "literal", value: strings[0]}])

    if (strings.length == 1) {
        return single_string_template
    }

    const s1_tokens = tokenize_string(strings[0])
    const s2_tokens = tokenize_string(strings[1])

    if (s1_tokens.length != s2_tokens.length) {
        return single_string_template
    }

    /** @type {Template} */
    const template = new Template(0, [])
    for (i = 0; i < s1_tokens.length; i++) {
        const s1_token = s1_tokens[i]
        const s2_token = s2_tokens[i]

        if (s1_token.type != s2_token.type) {
            return single_string_template
        }
        if (s1_token.type == "letter") {
            if (s1_token.value == s2_token.value) {
                template.parts.push({type: "literal", value: s1_token.value})
            } else {
                return single_string_template
            }
        } else {
            if (s1_token.value == s2_token.value) {
                template.parts.push({type: "literal", value: s1_token.value + ""})
            } else if (s1_token.value + 1 == s2_token.value) {
                template.parts.push({type: "range", min: s1_token.value, max: s2_token.value})
            } else {
                return single_string_template
            }
        }
    }

    template.count = 2
    for (const str of strings.slice(2)) {
        // template.add_string leaves the template in an inconsistent state if it fails
        const template_before = template.clone()
        const add_ok = template.add_string(str)
        if (!add_ok) {
            return template_before
        }
    }

    return template
}