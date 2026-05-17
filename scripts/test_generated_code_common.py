import subprocess
import os


def run_cmd(args, check_success=True):
    print(f"Running {args}")
    return_code = subprocess.call(args)
    if check_success:
        assert return_code == 0, f"Command exited with {return_code}"
    return return_code


def clone_or_update_repo(repo_url, clone_dir, commit):
    if not os.path.exists(clone_dir):
        run_cmd(['git', 'clone', '--depth=1', repo_url, clone_dir])
    else:
        run_cmd(['git', '-C', clone_dir, 'reset', '--hard'])
    run_cmd(['git', '-C', clone_dir, 'fetch', '--depth=1', 'origin', commit])
    run_cmd(['git', '-C', clone_dir, 'checkout', commit])
