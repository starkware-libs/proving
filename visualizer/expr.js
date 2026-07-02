/** @typedef {{get_html(view: any)}} Expr */

import { create_var_span, html } from "./utils.js"

export class BinaryOpNode {
    constructor(op, left_expr, right_expr) {
        this.left = left_expr
        this.right = right_expr
        this.op = op
    }
    
    static from_json(json, air) {
        const left = expr_from_json(json[0], air)
        const op = json[1]
        const right = expr_from_json(json[2], air)
        return new BinaryOpNode(op, left, right)
    }

    parenthesize_child(child) {
        if (!(child instanceof BinaryOpNode)) {
            return false
        }
        if (child instanceof BinaryOpNode) {
            if (this.op == '+' && child.op == '+') {
                return false
            }
            return true
        }
        throw `Unexpected child ${child}`
    }

    get_html(air_view, parenthesized) {
        if (parenthesized === undefined) {
            parenthesized = true
        }
        let left
        let right
        if (this.parenthesize_child(this.left)) {
            left = html`<span>(${this.left.get_html(air_view, true)})</span>`
        } else {
            left = this.left.get_html(air_view, false)
        }
        if (this.parenthesize_child(this.right)) {
            right = html`<span>(${this.right.get_html(air_view, true)})</span>`
        } else {
            right = this.right.get_html(air_view, false)
        }
        const result = html`<span>${left} ${this.op} ${right}</span>`
        result.addEventListener("mouseover", (e) => {
            result.style.background = "lightgray"
            if (parenthesized) {
                e.stopPropagation()
            }
        })
        result.addEventListener("mouseout", (e) => {
            result.style.background = "transparent"
        })
        return result
    }
}

export class VarNode {
    constructor(air, var_id) {
        this.air = air
        
        if (!air.vars.has(var_id)) {
            throw new Error(`Unknown var '${var_id}'`)
        }
        this.var = air.vars.get(var_id)
    }

    get_html(air_view) {
        return create_var_span(this.var, air_view)
    }
}

function state_cell_node(json, air) {
    const cell_id = json

    return new VarNode(air, cell_id)
}

function intermediate_var_node(json, air) {
    const [type, name] = json
    if (type != "M31") {
        throw new Error(`Unknown var type '${type}`)
    }

    return new VarNode(air, name)
}

export class ExternalCellNode {
    constructor(name, air) {
        this.table_name = name
    }

    get_html(air_view) {
        return html`<span>External::${this.table_name}</span>`
    }
}

export class PublicParamNode {
    constructor(json, air) {
        this.name = json
    }

    get_html(air_view) {
        return html`<span>PublicParam::${this.name}</span>`
    }
}

export class EnablerNode {
    get_html(air_view) {
        return html`<span>_enabler</span>`
    }
}

const MAX_FELT = 0xfffffffe
export class ConstNode {
    constructor(json, air) {
        if (json[0] != "M31") {
            throw `Unexpected const type ${json[0]}`
        }
        this.value = parseInt(json[1])
        if (this.value > MAX_FELT || this.value < 0) {
            throw `Invalid M31 value ${this.value}`
        }
    }

    is_power_of_2(x) {
        return (x > 0) && ((x & (x-1)) == 0)
    }

    get_html(air_view) {
        if (this.value > 4 && this.is_power_of_2(this.value)) {
            const log2 = Math.log2(this.value)
            return html`<span>2<sup>${log2}</sup></span>`
        }
        return html`<span>${this.value}</span>`
    }
}

/**
 * @param {Expr} expr 
 * @returns {Set<string>}
 */
export function expr_var_ids(expr) {
    if (expr instanceof BinaryOpNode) {
        const result = expr_var_ids(expr.left)
        for (const var_id of expr_var_ids(expr.right)) {
            result.add(var_id)
        }
        return result
    }
    if (expr instanceof VarNode) {
        return new Set([expr.var.id])
    }
    return new Set()
}

export function expr_from_json(json, air) {
    if (json.hasOwnProperty("BinaryOp")) {
        return BinaryOpNode.from_json(json.BinaryOp, air)
    }
    if (json.hasOwnProperty("State")) {
        return state_cell_node(json.State, air)
    }
    if (json.hasOwnProperty("Const")) {
        return new ConstNode(json.Const, air)
    }
    if (json.hasOwnProperty("Var")) {
        return intermediate_var_node(json.Var, air)
    }
    if (json.hasOwnProperty("ExternalState")) {
        return new ExternalCellNode(json.ExternalState, air)
    }
    if (json.hasOwnProperty("PublicParam")) {
        return new PublicParamNode(json.PublicParam, air)
    }
    if (typeof json == "string" && json == "Enabler") {
        return new EnablerNode()
    }
    throw new Error("Unknown expression node type " + Array.from(Object.keys(json)))
}