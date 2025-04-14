#!/usr/bin/env python3

"""
A web tool to show Stwo AIRs.

To use this tool, run this file and open the browser with the given URL.
"""

import http.server
import os
import socket
import socketserver
import sys
import urllib.parse
import glob
import json

VISUALIZER_DIR = os.path.abspath(os.path.dirname(__file__))
AIRS_DIR = os.path.join(VISUALIZER_DIR, "../crates/compiled_casm_air/src")

class MyTCPServer(socketserver.TCPServer):
    def server_bind(self):
        self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.socket.bind(self.server_address)


def main():

    # Run a simple http webserver.

    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args):
            super().__init__(*args, directory=VISUALIZER_DIR)

        def do_GET(self):
            parsed_path = urllib.parse.urlparse(self.path)
            if parsed_path.path == "/component_list":
                result = []
                for json_rel_path in glob.glob('*/*.json', root_dir=AIRS_DIR):
                    path = os.path.join(AIRS_DIR, json_rel_path)
                    air_name = json.load(open(path, 'rb'))['name']
                    result.append({"path": json_rel_path, "name": air_name})
                result_str = json.dumps(result)

                self.send_response(200)
                self.send_header("Content-type", "text/json")
                self.send_header("Content-Length", str(len(result_str)))
                self.end_headers()
                self.wfile.write(result_str.encode())
            elif parsed_path.path.startswith("/airs/"):
                air_json_rel_path = parsed_path.path.removeprefix("/airs/")
                air_json_path = os.path.join(AIRS_DIR, air_json_rel_path)
                size = os.path.getsize(air_json_path)
                self.send_response(200)
                self.send_header("Content-type", "text/json")
                self.send_header("Content-Length", str(size))
                self.end_headers()
                self.copyfile(open(air_json_path, 'rb'), self.wfile)
            else:
                super().do_GET()

    def start_server():
        port = 8000
        while True:
            try:
                return MyTCPServer(("localhost", port), Handler)
            except OSError:
                pass
            port += 1

    httpd = start_server()
    print("Running AIR Visualizer on http://localhost:%d/" % httpd.server_address[1])
    print()
    httpd.serve_forever()


if __name__ == "__main__":
    sys.exit(main())
