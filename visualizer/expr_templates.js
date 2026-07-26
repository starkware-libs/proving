import { BinaryOpNode, ConstNode, EnablerNode, ExternalCellNode, PublicParamNode, VarNode } from "./expr.js"
import { strings_maximal_template } from "./string_templates.js"
import { all_used_once, create_var_span, html, intersperse } from "./utils.js"

class TemplateVarNode {
    /**
     * @param {import("./air.js").Var[]} vars
     */
    constructor(vars) {
        const display_names = vars.map((v) => v.display_name)
        this.template = strings_maximal_template(display_names)
        if (this.template.count != vars.length) {
            throw new Error("Vars passed to TemplateVarNode are not a template")
        }
        this.vars = vars
    }

    get_expanded_html(air_view) {
        const clickable_vars = this.vars.map((v) => create_var_span(v, air_view))
        return html`<span class="expanded-template">${intersperse(clickable_vars, ", ")}</span>`
    }

    get_html(air_view) {
        if (this.vars.length == 1) {
            // A single var needs no collapse/expand affordance
            return create_var_span(this.vars[0], air_view)
        }

        let highlight_task = null
        const var_ids = this.vars.map((v) => v.id)
        const collapsed_elem = this.template.get_html(air_view, var_ids)
        const expanded_elem = this.get_expanded_html(air_view)

        // Prevent double-clicking on the collapsed template from selecting it
        collapsed_elem.addEventListener('mousedown', (e) => {
            if (e.detail > 1) {
                e.preventDefault()
            }
        })

        collapsed_elem.title = "Double-click to expand / collapse"
        if (all_used_once(this.vars, air_view)) {
            collapsed_elem.classList.add('used-once')
        }

        // For multiple-variable templates, expand / collapse on double-click
        collapsed_elem.addEventListener('dblclick', (e) => {
            if (highlight_task !== null) {
                clearTimeout(highlight_task)
                highlight_task = null
            }
            collapsed_elem.style.display = 'none'
            expanded_elem.style.display = 'inline'
        })
        expanded_elem.addEventListener('dblclick', (e) => {
            expanded_elem.style.display = 'none'
            collapsed_elem.style.display = 'inline'
        })

        collapsed_elem.addEventListener('click', (e) => {
            if (highlight_task !== null) {
                clearTimeout(highlight_task)
            }

            // Wait before highlighting. This click might be the first part of a double-click,
            // which means "expand", not "highlight".
            highlight_task = setTimeout(() => {
                air_view.select_vars(var_ids)
            }, 300)
        })

        // Start with the template collapsed
        expanded_elem.style.display = 'none'
        return html`${collapsed_elem}${expanded_elem}`
    }
}

class ExprTemplate {
    /**
     * @param {import("./expr.js").Expr} expr
     * @param {number} count
     */
    constructor(expr, count) {
        this.expr = expr
        this.count = count
    }

    get_html(air_view) {
        if (this.count == 1 || expr_has_ranges(this.expr)) {
            return this.expr.get_html(air_view)
        }

        if (this.count < 4) {
            const parts = intersperse(Array(this.count).fill(null).map(elem => this.expr.get_html(air_view)), ", ")
            console.log(this.count, parts)
            return html`${parts}`
        }
    
        return html`${this.expr.get_html(air_view)} <span class="template-range">× ${this.count}</span>`
    }
}

function expr_has_ranges(expr) {
    if (expr instanceof BinaryOpNode) {
        return expr_has_ranges(expr.left) || expr_has_ranges(expr.right)
    }

    if (expr instanceof TemplateVarNode) {
        return expr.template.parts.some(p => p.type == "range")
    }

    return false
}

/**
 * @param {import("./expr.js").Expr} expr1 
 * @param {import("./expr.js").Expr} expr2 
 * @returns {boolean}
 */
function structure_matches(expr1, expr2) {
    if (expr1 instanceof BinaryOpNode) {
        if (!(expr2 instanceof BinaryOpNode) || (expr2.op != expr1.op)) {
            return false
        }
        return structure_matches(expr1.left, expr2.left) && structure_matches(expr1.right, expr2.right)
    }

    if (expr1 instanceof ConstNode) {
        return (expr2 instanceof ConstNode) && expr1.value == expr2.value
    }

    if (expr1 instanceof ExternalCellNode) {
        return (expr2 instanceof ExternalCellNode) && 
                expr1.table_name == expr2.table_name
    }

    if (expr1 instanceof PublicParamNode) {
        return (expr2 instanceof PublicParamNode) && expr1.name == expr2.name
    }

    if (expr1 instanceof VarNode) {
        return (expr2 instanceof VarNode)
    }

    if (expr1 instanceof EnablerNode) {
        return (expr2 instanceof EnablerNode)
    }

    throw new Error(`Unexpected expression root type ${expr1}`)
}

/**
 * @param {import("./expr.js").Expr} expr
 * @returns {VarNode[]}
 */
function expr_var_nodes(expr) {
    if (expr instanceof BinaryOpNode) {
        return expr_var_nodes(expr.left).concat(expr_var_nodes(expr.right))
    }

    if (expr instanceof VarNode) {
        return [expr]
    }

    return []
}

/**
 * 
 * @param {import("./expr.js").Expr} expr 
 * @param {TemplateVarNode[]} new_var_nodes 
 * @returns {{result: import("./expr.js").Expr, replace_count: number}}
 */
function replace_var_nodes(expr, new_var_nodes) {
    if (expr instanceof BinaryOpNode) {
        const left_replace = replace_var_nodes(expr.left, new_var_nodes)
        const right_replace = replace_var_nodes(expr.right, new_var_nodes.slice(left_replace.replace_count))
        return {
            result: new BinaryOpNode(expr.op, left_replace.result, right_replace.result),
            replace_count: left_replace.replace_count + right_replace.replace_count
        }
    }

    if (expr instanceof VarNode) {
        return {result: new_var_nodes[0], replace_count: 1}
    }

    return {result: expr, replace_count: 0}
}

/**
 * @param {import("./expr.js").Expr[]} exprs
 * @returns {ExprTemplate}
 */
export function exprs_maximal_template(exprs) {
    let i
    if (exprs.length == 1) {
        return new ExprTemplate(exprs[0], 1)
    }

    let same_structure = 1
    while (same_structure < exprs.length) {
        if (structure_matches(exprs[0], exprs[same_structure])) {
            same_structure += 1
        } else {
            break
        }
    }

    exprs = exprs.slice(0, same_structure)

    const expr_0_var_nodes = expr_var_nodes(exprs[0])

    const var_node_instances = (new Array(expr_0_var_nodes.length)).fill(null).map((x) => [])
    for (const expr of exprs) {
        const var_nodes = expr_var_nodes(expr)
        if (var_nodes.length != var_node_instances.length) {
            throw new Error(`Expected ${var_node_instances.length} var nodes but got ${var_nodes.length}`)
        }

        for (i = 0; i < var_nodes.length; i++) {
            var_node_instances[i].push(var_nodes[i])
        }
    }

    let exprs_collected = exprs.length
    for (i = 0; i < var_node_instances.length; i++) {
        const node_names = var_node_instances[i].map((node) => node.var.display_name)
        const template = strings_maximal_template(node_names)
        if (template.count < exprs_collected) {
            exprs_collected = template.count
        }
    }

    exprs = exprs.slice(0, exprs_collected)
    let template_nodes = []
    for (i = 0; i < var_node_instances.length; i++) {
        const var_nodes = var_node_instances[i].slice(0, exprs_collected)
        const vars = var_nodes.map((n) => n.var)
        template_nodes.push(new TemplateVarNode(vars))
    }

    return new ExprTemplate(replace_var_nodes(exprs[0], template_nodes).result, exprs_collected)
}

/**
 * @param {import("./expr.js").Expr[]} exprs 
 * @returns {ExprTemplate[]}
 */
export function exprs_to_templates_greedy(exprs) {
    const result = []
    let exprs_processed = 0
    while (exprs_processed < exprs.length) {
        const maximal_template = exprs_maximal_template(exprs.slice(exprs_processed))
        result.push(maximal_template)
        exprs_processed += maximal_template.count
    }
    return result
}