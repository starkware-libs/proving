import { VarNode, expr_from_json, expr_var_ids } from "./expr.js"
import { exprs_maximal_template, exprs_to_templates_greedy } from "./expr_templates.js"
import { AirView, goto_air } from "./main.js"
import { strings_maximal_template } from "./string_templates.js"
import { DefaultMap, all_used_once, create_var_span, html, intersperse, remove_prefix, remove_suffix, shorten_name } from "./utils.js"


/** @interface */
class Step {
    /** Return the IDs of the variables that appear in this step
     * @returns {string[]}
     */
    get_var_ids() {
        throw new Error("Unimplemented");
    }

    /** Return the HTML that shows this step. Doesn't include
     * step number or step comments.
     * @param {AirView} air_view
     */
    get_html(air_view) {
        throw new Error("Unimplemented");
    }
}

/** @implements {Step} */
class ConstraintStep {
    constructor(json, air) {
        this.expr = expr_from_json(json[0], air)
        this.comment = json[1]
    }

    get_var_ids() {
        return Array.from(expr_var_ids(this.expr))
    }

    get_html(air_view) {
        return html`${this.expr.get_html(air_view)} = 0`
    }
}

/** @implements {Step} */
class IntermediateStep {
    constructor(json, air) {
        if (json.felt_names.length != 1) {
            throw new Error(`Unexpected intermediate of ${json.felt_names.length} felts`)
        }
        this.var_id = json.felt_names[0]
        this.air = air
        this.expr = expr_from_json(json.var, air)
    }

    get_var_ids() {
        const var_obj = this.air.vars.get(this.var_id)
        return Array.from(expr_var_ids(this.expr)).concat([var_obj.id])
    }

    get_html(air_view) {
        const var_obj = this.air.vars.get(this.var_id)
        return html`${create_var_span(var_obj, air_view)} = ${this.expr.get_html(air_view)}`
    }
}

/** @implements {Step} */
export class CallStep {
    constructor(json, air) {
        this.air = air
        this.air_name = remove_suffix(json.var.StaticCall[0], "::evaluate")
        this.output_var_ids = json.felt_names
        this.input_exprs = this.parse_input(json.var.StaticCall[1])
    }

    parse_input(input_json) {
        let input_exprs = []

        // The input might contain both felts and arrays of felts. Flatten it to one list of felts.
        for (const param of input_json) {
            if (param.hasOwnProperty("Array")) {
                for (const elem of param.Array) {
                    input_exprs.push(expr_from_json(elem, this.air))
                }
            } else {
                input_exprs.push(expr_from_json(param, this.air))
            }
        }

        return input_exprs
    }

    get_var_ids() {
        const var_ids = new Set(this.output_var_ids)
        for (const input_expr of this.input_exprs) {
            for (const var_id of expr_var_ids(input_expr)) {
                var_ids.add(var_id)
            }
        }
        return Array.from(var_ids)
    }

    get_html(air_view) {
        const param_templates = exprs_to_templates_greedy(this.input_exprs)
        const call_elem =
            html`<span class="air-link">${this.air_name}</span>
                        (${intersperse(param_templates.map(t => t.get_html(air_view)), ", ")})`
        call_elem[0].addEventListener('click', (e) => goto_air(this.air_name))

        if (this.output_var_ids.length == 0) {
            return call_elem
        } else {
            const output_var_objects = this.output_var_ids.map(name => this.air.vars.get(name))
            let output_var_html = html`${intersperse(output_var_objects.map(v => create_var_span(v, air_view)), ", ")}`

            const output_var_exprs = output_var_objects.map((v) => new VarNode(this.air, v.id))
            if (output_var_exprs.length > 1) {
                const template = exprs_maximal_template(output_var_exprs)
                if (template.count == output_var_objects.length) {
                    output_var_html = template.get_html(air_view)
                }
            }
            return html`[${output_var_html}] = ${call_elem}`
        }
    }
}

/** @implements {Step} */
export class LookupTermStep {
    /** @type {string} */
    relation_name
    /** @type {"Use" | "Yield"} */
    direction
    /** @type {import("./expr.js").Expr[]} */
    felts
    constructor(json, air) {
        this.relation_name = json.relation_name
        this.direction = json.use_or_yield
        this.felts = json.felts.map(x => expr_from_json(x, air))
    }

    get_var_ids() {
        const var_ids = new Set()
        for (const felt of this.felts) {
            for (const var_id of expr_var_ids(felt)) {
                var_ids.add(var_id)
            }
        }
        return Array.from(var_ids)
    }

    get_html(air_view) {
        const felt_templates = exprs_to_templates_greedy(this.felts)
        return html`${this.direction} ${this.relation_name}: [${intersperse(felt_templates.map(x => x.get_html(air_view)), ", ")}]`
    }
}

export function get_constraints_html(air, air_view) {
    const result = html`<div></div>`
    let step_num = 1
    for (const step of air.constraints) {
        const button = html`<span class="constraint-num-button">${step_num}</span>`
        button.addEventListener('click', (e) => air_view.select_vars(step.get_var_ids()))

        let comment = ""
        if (step instanceof ConstraintStep && step.comment !== null) {
            comment = html`<span class="constraint-comment">// ${step.comment}</span>`
        }
        result.appendChild(html`<div class="constraint">${comment}${button} ${step.get_html(air_view)}</div>`)
        step_num += 1
    }
    return result
}

class VarGroup {
    constructor(air, group_id, group_type, var_objs) {
        this.air = air
        this.var_objs = var_objs
        this.group_id = group_id
        this.group_type = group_type

        const var_display_names = this.var_objs.map(obj => obj.display_name)
        const template = strings_maximal_template(var_display_names)
        if (template.count == var_display_names.length) {
            this.template = template
            this.display_text = template.get_text()
        } else {
            throw new Error(`Cannot create VarGroup from variables ${var_display_names}`)
        }
    }

    get_title_html(air_view) {
        const var_ids = this.var_objs.map((obj) => obj.id)
        const result = this.template.get_html(air_view, var_ids)
        result.addEventListener('click', (e) => {
            air_view.select_vars(var_ids)
        })
        if (all_used_once(this.var_objs, air_view)) {
            result.classList.add('used-once')
        }
        return result
    }
}

export class Var {
    /**
     * @param {string} id The var's name in the JSON
     * @param {string} display_name How the var is displayed in the visualizer
     * @param {"input" | "state" | "intermediate"} type Is this var an input to this air? (if not, it is either an
     *                           intermediate variable or a state cell)
     */
    constructor(id, display_name, type) {
        this.id = id
        this.display_name = display_name
        this.group_id = undefined
        this.type = type
    }
}

export class Air {
    id

    // All the things we apply constraints to - state cells, intermediate variables and inputs
    /** @type {Map<string, Var>} */
    vars

    // A map from type to the IDs of all vars of that type
    vars_by_type

    // The order in which the vars in `this.vars` appear in the JSON. This is usually their "logical"
    // ordering.
    var_order

    // The name of the output relation for this AIR, if any
    relation_name

    var_groups
    group_order_by_type

    /** @type {Step[]} */
    constraints
    output_exprs
    output_names

    // A map from var ID to the number of expressions that reference it. Used to gray out
    // variables that are only referenced once.
    /** @type {Map<string, number>} */
    usage_counts

    constructor(json) {
        this.vars = new Map()
        this.vars_by_type = new DefaultMap(() => [])
        this.var_groups = new Map()
        this.group_order_by_type = new Map()
        this.var_order = []
        this.id = json.name
        this.relation_name = json.relation_name

        this.parse_state_and_input(json)
        this.collect_intermediate_vars(json)

        // At this point this.vars should contain all variables used in the constraints of this AIR

        this.disambiguate_display_names();

        // Do this after disambiguate_display_names so the slug is taken into account
        for (const var_type of ["input", "state", "intermediate"]) {
            this.group_order_by_type.set(var_type, this.create_var_groups(var_type))
        }

        this.constraints = this.parse_steps(json);

        [this.output_exprs, this.output_names] = this.parse_verifier_output(json)

        this.usage_counts = this.compute_usage_counts()
    }

    /**
     * Count, for each var ID, how many expressions reference it. Used to gray out variables
     * that are only referenced once.
     *
     * Inputs and call outputs are seeded with a baseline count of 1, since they're meaningful
     * interface/call boundaries: being declared/produced already counts as "1", so they only
     * show as referenced (count > 1, not grayed) once something actually uses them elsewhere.
     * Other vars (state cells, plain arithmetic intermediates) start at 0, so a single downstream
     * use is enough to gray them out.
     * @returns {Map<string, number>}
     */
    compute_usage_counts() {
        const counts = new Map()
        for (const step of this.constraints) {
            if (step instanceof CallStep) {
                for (const var_id of step.output_var_ids) {
                    counts.set(var_id, 1)
                }
            }
        }
        for (const var_id of this.vars_by_type.get("input")) {
            counts.set(var_id, 1)
        }

        const add_usages = (expr) => {
            for (const var_id of expr_var_ids(expr)) {
                counts.set(var_id, (counts.get(var_id) ?? 0) + 1)
            }
        }

        for (const step of this.constraints) {
            if (step instanceof ConstraintStep) {
                add_usages(step.expr)
            } else if (step instanceof IntermediateStep) {
                add_usages(step.expr)
            } else if (step instanceof CallStep) {
                for (const input_expr of step.input_exprs) {
                    add_usages(input_expr)
                }
            } else if (step instanceof LookupTermStep) {
                for (const felt of step.felts) {
                    add_usages(felt)
                }
            }
        }
        for (const output_expr of this.output_exprs) {
            add_usages(output_expr)
        }

        return counts
    }

    /**
     * @param {Var} var_obj
     */
    add_var(var_obj) {
        if (this.vars.has(var_obj.id)) {
            throw new Error(`Duplicate var id '${var_obj.id}'`)
        }
        this.vars.set(var_obj.id, var_obj)
        this.vars_by_type.get(var_obj.type).push(var_obj.id)
        this.var_order.push(var_obj.id)
    }

    disambiguate_display_names() {
        const display_name_counts = new DefaultMap(() => 0)

        // Count occurrences of each display name
        for (const var_id of this.var_order) {
            const display_name = this.vars.get(var_id).display_name

            display_name_counts.set(display_name, display_name_counts.get(display_name) + 1)
        }

        // Add slug ("#<number>") if necessary
        const counters = new DefaultMap(() => 0)

        for (const var_id of this.var_order) {
            const var_obj = this.vars.get(var_id)

            if (display_name_counts.get(var_obj.display_name) == 1) {
                // Unique name - doesn't need a counter
                continue
            }

            // Add slug to the display name
            const counter = counters.get(var_obj.display_name)
            counters.set(var_obj.display_name, counter + 1)
            var_obj.display_name = `${var_obj.display_name}#${counter}`
        }

    }

    /**
     * @param {string} group_type
     * @return {string[]}
     */
    create_var_groups(group_type) {
        let i
        const var_objs = this.vars_by_type.get(group_type).map((id) => this.vars.get(id))
        const group_order = []

        const display_names = var_objs.map(obj => obj.display_name)
        let vars_done = 0
        let id_counter = 0

        while (vars_done < var_objs.length) {
            const template = strings_maximal_template(display_names.slice(vars_done))
            const group_id = group_type + "" + id_counter
            const group = new VarGroup(this, group_id, group_type, var_objs.slice(vars_done, vars_done + template.count))
            if (this.var_groups.has(group_id)) {
                throw new Error(`Duplicate group ID ${group_id}`)
            }
            group_order.push(group_id)
            this.var_groups.set(group_id, group)
            vars_done += template.count
            id_counter++
        }

        const display_name_texts = Array.from(this.var_groups.values()).map(g => g.display_text)
        if ((new Set(display_name_texts)).size != display_name_texts.length) {
            throw new Error(`Duplicate group display names`)
        }

        return group_order
    }

    parse_state_and_input(json) {
        if (json.type == "Inline") {
            // Inline functions apply constraints to their inputs too
            const input_ids = json.verifier_input_limbs
            for (const input_id of input_ids) {
                this.add_var(new Var(input_id, shorten_name(input_id, this.id), "input"))
            }
        }

        let col = 0
        for (const cell_id of json.state_names) {
            const col_suffix = `_col${col}`
            const display_name = remove_suffix(cell_id, col_suffix)
            this.add_var(new Var(cell_id, shorten_name(display_name, this.id), "state"))
            col += 1
        }
    }

    collect_intermediate_vars(json) {
        for (const step of json.constraints) {
            if (step.hasOwnProperty("Intermediate")) {
                const created_felt_names = step.Intermediate.felt_names
                if (step.Intermediate.var.hasOwnProperty("StaticCall")) {
                    for (const var_id of created_felt_names) {
                        this.add_var(new Var(var_id, shorten_name(var_id, this.id), "intermediate"))
                    }
                } else {
                    // Not a call - we expect a single felt value
                    if (created_felt_names.length != 1) {
                        throw new Error(`Non-call intermediate creates ${created_felt_names.length} felts`)
                    }
                    this.add_var(new Var(created_felt_names[0], shorten_name(created_felt_names[0], this.id), "intermediate"))
                }
            }
        }
    }

    parse_steps(json) {
        const result = []
        for (const step of json.constraints) {
            if (step.hasOwnProperty("Constraint")) {
                result.push(new ConstraintStep(step.Constraint, this))
                continue
            }
            if (step.hasOwnProperty("Intermediate")) {
                if (step.Intermediate.var.hasOwnProperty("StaticCall")) {
                    result.push(new CallStep(step.Intermediate, this))
                } else {
                    result.push(new IntermediateStep(step.Intermediate, this))
                }
                continue
            }
            if (step.hasOwnProperty("LookupTerm")) {
                result.push(new LookupTermStep(step.LookupTerm, this))
                continue
            }
            throw new Error("Unknown step type")
        }
        return result
    }

    parse_verifier_output(json) {
        const [output, output_felt_names] = json.verifier_output

        const output_felt_name_prefix = `${json.name}_`
        const output_felt_display_names = output_felt_names
                                          .map((n) => remove_prefix(n, output_felt_name_prefix))
        return [output.Array.map(x => expr_from_json(x, this)), output_felt_display_names]
    }
}
