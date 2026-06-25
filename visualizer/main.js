import { Air, CallStep, get_constraints_html, LookupTermStep } from "./air.js"
import { DefaultMap, create_var_span, html, intersperse, zip } from "./utils.js"

let AIRS = new Map()
let RELATIONS = new DefaultMap(() => { return { used_by: new Set() } })
let selected_air = null

class VarGroupView {
    constructor(air_view, group_id) {
        this.group_id = group_id
        this.expanded = false

        this.button = document.createElement("span")
        this.button.innerText = "+"
        this.button.addEventListener("click", (event) => this.expand(event))

        this.var_list = document.createElement("div")
        const group = air_view.air.var_groups.get(this.group_id)
        for (const var_obj of group.var_objs) {
            const var_div = document.createElement("div")
            var_div.append(create_var_span(var_obj, air_view))
            var_div.style.marginLeft = "20px"
            this.var_list.append(var_div)
        }
        this.var_list.style.display = "none"


        this.element = document.createElement("div")

        let title_elem
        if (group.var_objs.length > 1) {
            this.element = html`
                <div>
                    ${this.button}<span> ${group.get_title_html(air_view)}</span>
                    ${this.var_list}
                </div>`
        } else {
            title_elem = create_var_span(group.var_objs[0], air_view)
            this.element.append(title_elem)
        }
    }

    expand(event) {
        if (this.expanded == false) {
            this.expanded = true
            this.button.innerText = "\u2014"
            this.var_list.style.display = "block"
        } else {
            this.expanded = false
            this.button.innerText = "+"
            this.var_list.style.display = "none"
        }
    }

}

class VarsView {
    constructor(air_view, group_ids) {
        let i
        this.element = html`<div style="display: flex; width: 100%"></div>`
        const columns = []
        for (i = 0; i < 3; i++) {
            const column = html`<div style="flex-grow: 1"></div>`
            columns.push(column)
            this.element.append(column)
        }

        const groups_per_column = Math.max(5, Math.ceil(group_ids.length / columns.length))

        i = 0
        for (const group_id of group_ids) {
            const group_view = new VarGroupView(air_view, group_id)
            columns[Math.floor(i / groups_per_column)].appendChild(group_view.element)
            i += 1
        }
    }
}

export class AirView {
    /** @param {Air} [air] */
    constructor(air) {
        let i
        this.var_select_listeners = []
        this.selected_var_ids = []
        this.air = air

        this.inputs_view = new VarsView(this, this.air.group_order_by_type.get("input"))
        this.cells_view = new VarsView(this, this.air.group_order_by_type.get("state"))
        this.internal_view = new VarsView(this, this.air.group_order_by_type.get("intermediate"))
    }

    select_vars(var_ids) {
        for (const id of var_ids) {
            if (this.selected_var_ids.includes(id)) {
                var_ids = []
                break
            }
        }
        this.selected_var_ids = var_ids
        for (const func of this.var_select_listeners) {
            func(this.selected_var_ids)
        }
    }
}

function populate_component_selector() {
    const select_elem = document.getElementById("component_select")
    const by_group = new Map()
    for (const air_name of Array.from(AIRS.keys()).sort()) {
        const group = AIRS.get(air_name).group || ""
        if (!by_group.has(group)) by_group.set(group, [])
        by_group.get(group).push(air_name)
    }
    for (const [group, names] of by_group) {
        const optgroup = document.createElement('optgroup')
        optgroup.label = group
        for (const air_name of names) {
            const option = document.createElement('option')
            option.innerText = air_name
            option.value = air_name
            optgroup.append(option)
        }
        select_elem.append(optgroup)
    }
}

async function build_xref_graph() {
    let done = 0
    for (const air_name of AIRS.keys()) {
        const json = await get_air_json(air_name)
        for (const lookup of json.constraint_lookups) {
            const [relation_name, use_or_yield] = lookup
            if (use_or_yield == "Use") {
                RELATIONS.get(relation_name).used_by.add(air_name)
            }
        }

        for (const callee in json.inline_calls) {
            AIRS.get(callee).called_from.add(air_name)
        }
        set_error(`Building xref graph: ${done} / ${AIRS.size}`)
        done++
    }
}

async function init() {
    const response = await fetch("/component_list")
    const json = await response.json()
    for (const air of json) {
        AIRS.set(air.name, {...air, comment: "", called_from: new Set()})
    }
    await build_xref_graph()
    await update_comments()
    setInterval(update_comments, 1000)
    populate_component_selector()
    addEventListener('popstate', (e) => show_air(e.state.air_name))
    document.getElementById("component_select").addEventListener('change', component_select_change)
    document.getElementById("xrefs_panel_toggle").addEventListener('click', toggle_xrefs_panel)
    await goto_air('add_252')
}

function set_error(msg) {
    document.getElementById("error").innerText = msg
}

async function update_comments() {
    const decoder = new TextDecoder()
    let response
    try {
        response = await fetch("/comments")
    } catch (e) {
        set_error(`Cannot load comments: ${e}`)
        return
    }
    const text = decoder.decode(await response.arrayBuffer())
    const lines = text.split(/\n/).map(s => s.trim())

    const comments = new Map()
    let cur_air = null
    let cur_comment = ""
    for (const line of lines) {
        if (line.slice(0,2) == '# ') {
            const air_name = line.slice(2).trim()
            if (!AIRS.has(air_name)) {
                set_error(`Comments file contains unknown AIR name ${air_name}`)
                return
            }
            cur_comment = cur_comment.trim()
            if (cur_air !== null) {
                AIRS.get(cur_air).comment = cur_comment
            }
            cur_comment = ""
            cur_air = air_name
        } else {
            cur_comment += "\n" + line
        }
    }
    if (cur_air !== null) {
        AIRS.get(cur_air).comment = cur_comment
    }

    set_error("")
    show_comment()
}

async function get_air_json(air_name) {
    const json_path = AIRS.get(air_name).path
    const url = `airs/${json_path}`
    const response = await fetch(url)
    return await response.json()
}

export async function goto_air(air_name) {
    if (air_name == "") {
        return
    }
    history.pushState({'air_name': air_name}, "")
    show_air(air_name)
}

function decode_instruction_inst_def_html(instance_definition) {
    for (const key in instance_definition) {
        if (key != "const_offsets" && key != "const_flags" && key != "const_opcode_extension" && key != "flag_sets_of_sum_1") {
            throw new Error(`Unexpected decode_instruction instance definition key ${key}`)
        }
    }

    const const_flags = []
    const flag_names = Array.from(Object.keys(instance_definition.const_flags)).sort()
    for (const flag of flag_names) {
        const_flags.push({"name": flag, "value": instance_definition.const_flags[flag]})
    }

    return html`<div>
        Const offsets: ${intersperse(instance_definition.const_offsets.map(x => x ?? "?"), ", ")}<br/>
        Flags: <br/>${
            const_flags.map(cf => html`<div style="margin-left: 20px">${cf.name} = ${cf.value ?? "?"}</div>`)
        }
        Opcode extension: ${instance_definition.const_opcode_extension}<br/>
        Flag sets that sum to 1: ${instance_definition.flag_sets_of_sum_1.length > 0 ? 
            instance_definition.flag_sets_of_sum_1 + "" : "(none)"}
    </div>`
}

function show_comment() {
    if (selected_air === null) {
        return
    }
    const comment = AIRS.get(selected_air).comment
    document.getElementById("air_comment").innerHTML = ""
    if (comment != "") {
        document.getElementById("air_comment").style.display = "block"
        document.getElementById("air_comment").innerText = `/*\n${comment}\n*/`
    } else {
        document.getElementById("air_comment").style.display = "none"
    }
}

async function show_air(air_name) {
    selected_air = air_name

    const json = await get_air_json(air_name)
    const air = new Air(json)

    document.getElementById("inst_def_panel").innerHTML = ""
    if (air_name.match(/^decode_instruction_[0-9a-f]+$/)) {
        document.getElementById("inst_def_panel").append(
            decode_instruction_inst_def_html(JSON.parse(json.instance_definition))
        )
    } 

    (/** @type {HTMLSelectElement} */ (document.getElementById("component_select"))).value = air_name

    const view = new AirView(air)
    document.getElementById("inputs_panel").replaceChildren(view.inputs_view.element)
    document.getElementById("cells_panel_title").innerText = json.type == "Inline" ? "State cells (from caller)" : "State cells"
    document.getElementById("cells_panel").replaceChildren(view.cells_view.element)
    document.getElementById("vars_panel").replaceChildren(view.internal_view.element)
    document.getElementById("constraints_panel").replaceChildren(get_constraints_html(air, view))
    show_comment()

    fill_xrefs_panel(air)

    let outputs_html = []
    let output_idx = 0
    for (const [output_expr, output_expr_name] of zip(air.output_exprs, air.output_names)) {
        outputs_html.push(html`<div>output[${output_idx}] (${output_expr_name}) = ${output_expr.get_html(view)}</div>`)
        output_idx += 1
    }
    document.getElementById("output_panel").replaceChildren(...outputs_html)
}

async function component_select_change() {
    const selected = (/** @type {HTMLSelectElement} */ (document.getElementById("component_select"))).value
    await goto_air(selected)
}

function air_fn_list_html(air_fn_names) {
    let result = []
    for (const fn_name of Array.from(air_fn_names).sort()) {
        const line_elem = html`<div class="air-link" style="margin-top: 2px; margin-bottom: 2px">${fn_name}</div>`
        line_elem.addEventListener('click', (e) => goto_air(fn_name))
        result.push(line_elem)
    }
    return result
}

/**
 * @param {Air} air
 */
function fill_xrefs_panel(air) {
    let xrefs_panel_content = document.getElementById("xrefs_panel_content")
    xrefs_panel_content.replaceChildren(html`<div style="font-family: sans-serif; font-weight: bold;">Called by</div>`)
    xrefs_panel_content.appendChild(html`<div>${air_fn_list_html(AIRS.get(air.id).called_from)}</div>`)

    if (air.relation_name !== null) {
        xrefs_panel_content.appendChild(
            html`<div style="font-family: sans-serif; font-weight: bold; margin-top: 15px">Relation <span class="code-font">${air.relation_name}</span> used by</div>`
        )
        xrefs_panel_content.appendChild(html`<div>${air_fn_list_html(RELATIONS.get(air.relation_name).used_by)}</div>`)
    }
}

async function toggle_xrefs_panel() {
    const button = document.getElementById("xrefs_panel_toggle")
    const panel = document.getElementById("xrefs_panel")
    const panel_content = document.getElementById("xrefs_panel_content")
    const panel_teaser = document.getElementById("xrefs_panel_teaser")
    if (panel.classList.contains("panel-open")) {
        // The panel is open, close it
        button.innerText = "<"
        panel.classList.remove("panel-open")
        panel.classList.add("panel-closed")
        panel_content.style.display = "none"
        panel_teaser.style.display = "block"
    } else {
        // The panel is closed, open it
        button.innerText = ">"
        panel.classList.remove("panel-closed")
        panel.classList.add("panel-open")
        panel_content.style.display = "block"
        panel_teaser.style.display = "none"
    }
}

init()
