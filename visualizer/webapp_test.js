// Headless-browser smoke test for the visualizer web-app.
//
// Starts visualizer.py, loads the page in headless Chrome, waits for the default component to
// render, and asserts the app populated the UI with no console/page errors. Also asserts every
// component's AIR JSON is servable, so this single test covers both the front-end and the
// back-end.
//
// Run via scripts/visualizer_webapp_test.sh — it needs Python, a browser, and extra permissions.

import { launch } from "jsr:@astral/astral@0.5.3"
import { strict as assert } from "node:assert"

// visualizer.py serves on port 8000 by default, and CI runs on a clean machine where it is free.
const URL = "http://localhost:8000/"
// ubuntu-latest ships google-chrome preinstalled and on PATH; astral launches it by name.
const CHROME = "google-chrome"

// visualizer.py has no readiness signal we can listen for, so poll the page until it answers.
async function wait_for_server() {
    for (let i = 0; i < 50; i++) {
        try {
            await (await fetch(URL)).body?.cancel()
            return
        } catch {
            await new Promise((resolve) => setTimeout(resolve, 100))
        }
    }
    throw new Error(`visualizer.py did not come up on ${URL}`)
}

Deno.test({
    name: "visualizer web-app renders a component without errors",
    // This is an integration test that spawns a subprocess and a browser; opt out of the
    // resource/op sanitizers that those long-lived handles would otherwise trip.
    sanitizeResources: false,
    sanitizeOps: false,
    fn: async () => {
        // Inherit the server's stdout/stderr so a startup failure or traceback shows in the log.
        const server = new Deno.Command("python3", {
            args: ["./visualizer/visualizer.py"],
            stdout: "inherit",
            stderr: "inherit",
        }).spawn()
        const browser = await launch({ headless: true, path: CHROME, args: ["--no-sandbox"] })
        try {
            await wait_for_server()
            const page = await browser.newPage()
            const errors = []
            page.addEventListener("console", (e) => {
                if (e.detail.type === "error") errors.push(e.detail.text)
            })
            page.addEventListener("pageerror", (e) => errors.push(String(e.detail)))

            await page.goto(URL, { waitUntil: "load" })

            // init() populates the component selector and renders the default component
            // into the constraints panel.
            await page.waitForSelector("#component_select option")
            await page.waitForSelector("#constraints_panel > *")

            const component_count = await page.evaluate(
                "document.querySelectorAll('#component_select option').length",
            )
            const error_banner = await page.evaluate(
                "document.getElementById('error').innerText",
            )

            assert.ok(component_count > 0, "component selector should be populated")
            assert.equal(error_banner, "", `visualizer reported an error: ${error_banner}`)
            assert.deepEqual(errors, [], `console/page errors: ${errors.join(" | ")}`)

            // Back-end coverage: every component's AIR JSON must be servable (the browser only
            // renders the default one above).
            const components = await (await fetch(`${URL}component_list`)).json()
            assert.ok(
                Array.isArray(components) && components.length > 0,
                "/component_list should return components",
            )
            for (const { path } of components) {
                const res = await fetch(`${URL}airs/${path}`)
                await res.body?.cancel()
                assert.ok(res.ok, `/airs/${path} returned ${res.status}`)
            }
        } finally {
            await browser.close()
            server.kill()
            await server.status
        }
    },
})
