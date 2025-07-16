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
import argparse

VISUALIZER_DIR = os.path.abspath(os.path.dirname(__file__))
AIRS_DIR = os.path.join(VISUALIZER_DIR, "../crates/compiled_casm_air/src")

class MyTCPServer(socketserver.TCPServer):
    def server_bind(self):
        self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.socket.bind(self.server_address)

def get_air_files():
    result = []
    for json_rel_path in glob.glob('*/*.json', root_dir=AIRS_DIR):
        path = os.path.join(AIRS_DIR, json_rel_path)
        air_name = json.load(open(path, 'rb'))['name']
        result.append({"path": json_rel_path, "name": air_name})
    return result

def main():

    # Run a simple http webserver.

    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args):
            super().__init__(*args, directory=VISUALIZER_DIR)

        def do_GET(self):
            parsed_path = urllib.parse.urlparse(self.path)
            if parsed_path.path == "/comments":
                if commandline.comment_file is None:
                    contents = ""
                else:
                    f = open(commandline.comment_file)
                    contents = f.read()
                    f.close()

                self.send_response(200)
                self.send_header("Content-Type", "text/markdown")
                self.send_header("Content-Length", len(contents))
                self.end_headers()
                self.wfile.write(contents.encode())
            elif parsed_path.path == "/component_list":
                result = []

                result_str = json.dumps(get_air_files())

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

    parser = argparse.ArgumentParser()
    parser.add_argument("-c", "--comment-file")
    commandline = parser.parse_args()

    if commandline.comment_file is not None and not os.path.exists(commandline.comment_file):
        print(f"Error: Comments file not found. Creating.")
        with open(commandline.comment_file, "w") as f:
            f.write("\nAIR comments file\n\nAdd comments for each AIR in the lines after its name\n\n")
            air_names = [x["name"] for x in get_air_files()]
            for name in sorted(air_names):
                f.write(f"# {name}\n\n")

    httpd = start_server()
    print("Running AIR Visualizer on http://localhost:%d/" % httpd.server_address[1])
    print()
    httpd.serve_forever()


if __name__ == "__main__":
    sys.exit(main())
