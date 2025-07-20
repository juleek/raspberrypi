#!/bin/python3

import os
import sys
import logging
import argparse
import subprocess
import typing as t
import pathlib as pl
import secret
import dataclasses as dc

#
# ============================================================================================================
# logger

logging.basicConfig(format='%(asctime)s %(levelname)s %(funcName)s:%(lineno)d: %(message)s')

# Exit on critical log
class ShutdownHandler(logging.Handler):
    def emit(self, record):
        print(record, file=sys.stderr)
        logging.shutdown()
        sys.exit(1)
logger = logging.getLogger()
logger.addHandler(ShutdownHandler(level=logging.CRITICAL))



#
# ============================================================================================================
# shell command running



@dc.dataclass
class ExecRes:
    out: str = ""
    ret: int = 0
    def is_ok(self) -> bool:
        return self.ret == 0
    def is_err(self) -> bool:
        return not self.is_ok()

def exec(dry_run: bool, command: str, echo_output: t.Union[bool, None] = None, root_is_required = False, input: t.Optional[str] = None) -> ExecRes:
    # if echo_output == None:
    #     echo_output=logger.level <= logging.DEBUG
    if root_is_required and os.geteuid() != 0:
        command = "sudo " + command
    if echo_output:
        logger.info(f'{"NOT " if dry_run else ""}executing: {command}')
    else:
        logger.debug(f'{"NOT " if dry_run else ""}executing: {command}')
    if dry_run:
        return ExecRes()

    try:
        process = subprocess.Popen(command,
                                   shell=True,
                                   stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT,
                                   text=True,
                                   stdin=subprocess.PIPE if input is not None else None)
    except Exception as e:
        return ExecRes(out=f'Failed to start new process: {command}: {e}', ret=1)

    output = ""

    if input is not None:
        try:
            process.stdin.write(input)
            process.stdin.close()
        except Exception as e:
            process.terminate()
            return ExecRes(out=f'Failed to write stdin: {e}', ret=1)

    for line in iter(process.stdout.readline, ''):
        output += line
        if echo_output:
            sys.stdout.write(line)
            sys.stdout.flush()

    ret = process.wait()
    output = output.rstrip()
    process.stdout.close()

    return ExecRes(out=output, ret=ret)




#
# ============================================================================================================
# systemctl units

def tls_dir(user: str) -> pl.Path:
    return pl.Path("/home") / user / "tls"


def src_root(user: str) -> pl.Path:
    return pl.Path("/home/") / user / "raspberrypi" / 'thermo_rust'


def this_file() -> pl.Path:
    return pl.Path(__file__).expanduser().resolve()


def src_root_rel_to_script() -> pl.Path:
    return this_file().parent


def git_pull_and_get_changed(dry_run: bool, repo_path: pl.Path, target_paths: t.List[pl.Path]) -> bool:
    import shlex
    def get_hashes(target_paths: t.List[pl.Path]) -> t.Dict[pl.Path, str]:
        hashes = {}
        for path in target_paths:
            cmd_before = f"cd {shlex.quote(str(repo_path))} && git log -1 --format=%H -- {shlex.quote(str(path))}"
            res_before = exec(dry_run=False, command=cmd_before, echo_output=False)
            if res_before.is_err():
                logger.critical(f"Failed to get commit before pull for {path}: {res_before.out}")
            hashes[path] = res_before.out.strip()
        return hashes


    hashes_before = get_hashes(target_paths)

    # Perform git pull
    cmd_pull = f"cd {shlex.quote(str(repo_path))} && git pull"
    res_pull = exec(dry_run=dry_run, command=cmd_pull, echo_output=False)
    if res_pull.is_err():
        logger.critical(f"Git pull failed: {res_pull.out}")


    hashes_after = get_hashes(target_paths)
    res = hashes_before != hashes_after
    if res == True:
        logger.info(f"Changes in one of target dirs are detected: before: {hashes_before}, after: {hashes_after}")
    else:
        logger.info(f"No changes in one of target dirs are detected: before and after: {hashes_before}")
    return res



def systemd_setup_3g_4g_on_boot_timer() -> t.Tuple[str, str]:
    return (f"""
[Unit]
Description=timer unit for enabling network only on boot

[Timer]
OnBootSec=60
Unit=setup_3g_4g.service

[Install]
WantedBy=multi-user.target
""", "setup_3g_4g_on_boot.timer")




def systemd_setup_3g_4g_service() -> t.Tuple[str, str]:
    return (f"""
[Unit]
Description=(re-)enable 3g/4g modem internet
Requires=network-online.target
After=network-online.target

[Service]
Type=oneshot
User=root
ExecStart=-/sbin/dhclient -v usb0
ExecStart=-/usr/bin/curl -v --header "Referer: http://192.168.0.1/index.html" http://192.168.0.1/goform/goform_set_cmd_process?goformId=CONNECT_NETWORK
ExecStart=-/usr/bin/curl -v --header "Referer: http://192.168.0.1/index.html" http://192.168.0.1/goform/goform_set_cmd_process?goformId=SET_CONNECTION_MODE&ConnectionMode=auto_dial&roam_setting_option=on
Restart=no

[Install]
WantedBy=multi-user.target
""", "setup_3g_4g.service")




def systemd_setup_3g_4g_timer() -> t.Tuple[str, str]:
   return (f"""
[Unit]
Description=timer unit for updating network periodically
Requires=network-online.target
After=network-online.target

[Timer]
OnCalendar=*-*-* *:00/10:00
Unit=setup_3g_4g.service

[Install]
WantedBy=multi-user.target
""", "setup_3g_4g.timer")




def systemd_main_service(full_cmd_line: pl.Path, user: str) -> t.Tuple[str, str]:
    return (f"""
[Unit]
Description=thermo daemon
Requires=network-online.target
After=network-online.target


[Service]
Type=simple
User={user}
ExecStart={full_cmd_line}
Restart=always
RestartSec=60

[Install]
WantedBy=multi-user.target
""", "thermo.service")




def systemd_update_service(package: str, user: str) -> t.Tuple[str, str]:
    return (f"""
[Unit]
Description=service for updating repo with thermo project and installing it in the system
Requires=network-online.target
After=network-online.target

[Service]
Type=simple
User={user}
ExecStart={this_file()} install --{package}
Restart=no

[Install]
WantedBy=multi-user.target
""", "update_thermo.service")




def systemd_update_timer() -> t.Tuple[str, str]:
    return (f"""
[Unit]
Description=timer unit for updating repo with thermo project and installing it in the system
Requires=network-online.target
After=network-online.target

[Timer]
OnCalendar=*-*-* *:18:00
# OnCalendar=*-*-* *:0/5:00
Unit=update_thermo.service

[Install]
WantedBy=multi-user.target
""", "update_thermo.timer")




def same_content(old: pl.Path, new_content: str) -> bool:
    if not old.exists():
        return False
    try:
        old_content = old.read_text(encoding='utf-8')
    except Exception as e:
        logger.critical(f"Failed to read from {old}: {e}")
    return old_content == new_content

def systemd_unit_path(filename: str) -> pl.Path:
    return pl.Path(f"/etc/systemd/system") / filename

def install_system_systemd_unit(content_name: t.Tuple[str, str], restart: bool, dry_run: bool):
    content, service_name = content_name
    service_path: pl.Path = systemd_unit_path(service_name)

    if same_content(service_path, content):
        logger.info(f"No changes in service {service_path} => skipping it")
        return

    logger.info(f"Installing unit: {service_name}")

    return

    res: ExecRes = exec(dry_run=dry_run, command=f"tee {service_path}", root_is_required=True, input=content)
    if res.is_err():
        logger.critical(f"Failed to copy & write contents of {service_name}: {res}")

    res: ExecRes = exec(dry_run=dry_run, command="systemctl daemon-reload", root_is_required=True)
    if res.is_err():
        logger.critical(f"Failed to reload systemd daemon: {res}")

    if not restart:
        return

    res: ExecRes = exec(dry_run=dry_run, command=f"systemctl enable {service_name}", root_is_required=True)
    if res.is_err():
        logger.critical(f"Failed to enable service: {res}")

    res: ExecRes = exec(dry_run=dry_run, command=f"systemctl restart {service_name}", root_is_required=True)
    if res.is_err():
        logger.critical(f"Failed to restart service: {res}")



#
# ============================================================================================================
# common helpers

def cargo_path() -> pl.Path:
    return pl.Path("~/.cargo/bin/cargo").expanduser()

def install_rust_if_needed(dry_run: bool):
    # Check if Cargo is installed
    check_command = f"{cargo_path()} --version"
    check_res: ExecRes = exec(dry_run=dry_run, command=check_command)

    if not check_res.is_err():
        logger.info("Rust is already installed.")
        return

    logger.info("Rust not found. Installing...")

    command = "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    res: ExecRes = exec(dry_run=dry_run, command=command)
    if res.is_err():
        logger.critical(f"Failed to install Rust: {res}")

    # Verify that Rust is installed
    verify_res: ExecRes = exec(dry_run=dry_run, command=check_command)
    if verify_res.is_err():
        logger.critical("Rust installation failed: Cargo not found after installation.")

    logger.info("Rust installed successfully.")



# executes cargo build --release -p {package} and returns the path with the result of compilation
def build_package(package: str, src_root: pl.Path, dry_run: bool) -> pl.Path:
    command: str = f"{cargo_path()} build --manifest-path {src_root / 'Cargo.toml'} --release -p {package}"
    logger.info(f"Building: {package} via: {command}")
    res: ExecRes = exec(dry_run=dry_run, command=command, echo_output=True)
    if res.is_err():
        logger.critical(f"Failed to build package '{package}': {res}")

    res: pl.Path = src_root / "target" / "release" / package
    logger.info(f"Built {package} at: {res}")
    return res


#
# ============================================================================================================
# generate TLS certificates

def generate_tls(out_dir: pl.Path, src_root: pl.Path, dry_run: bool):
   install_rust_if_needed(dry_run)
   server: pl.Path = build_package("server", src_root, dry_run)

   command: str = f"{server} tls ca --ca-cert {out_dir / 'ca.cert'} --ca-key {out_dir / 'ca.key'}"
   res: ExecRes = exec(dry_run=dry_run, command=command)
   if res.is_err():
       logger.critical(f"Failed to generate ca: {command}: {res}")

   command: str = " ".join([
       f"{server}"                                  ,
       f"tls server --ca-cert {out_dir / 'ca.cert'}",
       f"--ca-key {out_dir / 'ca.key'}"             ,
       f"--cert {out_dir / 'server.cert'}"          ,
       f"--key {out_dir / 'server.key'}"            ,
       f"--san-ips {secret.SERVER_IP}"])
   res: ExecRes = exec(dry_run=dry_run, command=command)
   if res.is_err():
       logger.critical(f"Failed to generate server: {command}: {res}")

   command: str = " ".join([
       f"{server}"                                  ,
       f"tls client --ca-cert {out_dir / 'ca.cert'}",
       f"--ca-key {out_dir / 'ca.key'}"             ,
       f"--cert {out_dir / 'client.cert'}"          ,
       f"--key {out_dir / 'client.key'}"])
   res: ExecRes = exec(dry_run=dry_run, command=command)
   if res.is_err():
       logger.critical(f"Failed to generate client: {command}: {res}")



#
# ============================================================================================================
# installation & update

def install_client(dry_run: bool):
   install_rust_if_needed(dry_run)
   user: str = secret.USER_ON_RPI

   src_code_dirs: t.List[pl.Path] = [src_root_rel_to_script()/"common", src_root_rel_to_script()/"sensor", src_root_rel_to_script()/"server"]
   if git_pull_and_get_changed(dry_run, src_root_rel_to_script(), src_code_dirs):
      sensor: pl.Path = build_package("sensor", src_root_rel_to_script(), dry_run)

   install_system_systemd_unit(systemd_main_service(" ".join([
       f"{sensor}"                                                ,
       f"--server-host-port {secret.SERVER_IP}:{secret.GRPC_PORT}",
       f"--bottom-id {secret.BOTTOM_ID}"                          ,
       f"--bottom-path {secret.BOTTOM_PATH}"                      ,
       f"--ambient-id {secret.AMBIENT_ID}"                        ,
       f"--ambient-path {secret.AMBIENT_PATH}"                    ,
       f"--tls-ca-cert {tls_dir(user) / 'ca.cert'}"               ,
       f"--tls-client-cert {tls_dir(user) / 'client.cert'}"       ,
       f"--tls-client-key {tls_dir(user) / 'client.key'}"         ,
    ]), user), restart=True, dry_run=dry_run)
   install_system_systemd_unit(systemd_update_service("sensor", user), restart=False, dry_run=dry_run)
   install_system_systemd_unit(systemd_update_timer(), restart=True, dry_run=dry_run)

   install_system_systemd_unit(systemd_setup_3g_4g_service(), restart=False, dry_run=dry_run)
   install_system_systemd_unit(systemd_setup_3g_4g_timer(), restart=True, dry_run=dry_run)
   install_system_systemd_unit(systemd_setup_3g_4g_on_boot_timer(), restart=True, dry_run=dry_run)



def install_server(dry_run: bool):
   install_rust_if_needed(dry_run)

   src_code_dirs: t.List[pl.Path] = [src_root_rel_to_script()/"common", src_root_rel_to_script()/"sensor", src_root_rel_to_script()/"server"]
   if git_pull_and_get_changed(dry_run, src_root_rel_to_script(), src_code_dirs):
      server: pl.Path = build_package("server", src_root_rel_to_script(), dry_run)
      res: ExecRes = exec(dry_run=dry_run, command=f"cp {server} {secret.SERVER_PATH}", root_is_required=True)
      if res.is_err():
          logger.critical(f"Failed to build server: {res}")
      res: ExecRes = exec(dry_run=dry_run, command=f"systemctl restart thermo.service", root_is_required=True)
      if res.is_err():
          logger.critical(f"Failed to restart service: {res}")


   user: str = secret.USER_ON_SRV
   install_system_systemd_unit(systemd_main_service(" ".join([
       f"{secret.SERVER_PATH} serve"                       ,
       f"--host-port 0.0.0.0:{secret.GRPC_PORT}"           ,
       f"--db-path {secret.DB_PATH}"                       ,
       f"--tls-ca-cert {tls_dir(user) / 'ca.cert'}"        ,
       f"--tls-server-cert {tls_dir(user) / 'server.cert'}",
       f"--tls-server-key {tls_dir(user) / 'server.key'}"  ,
   ]), user), restart=True, dry_run=dry_run)
   install_system_systemd_unit(systemd_update_service("server", secret.SUDO_USER_ON_SRV), restart=False, dry_run=dry_run)
   install_system_systemd_unit(systemd_update_timer(), restart=True, dry_run=dry_run)




#
# ============================================================================================================
# main


def main():
    ARG_TLS: str = "tls"
    ARG_INSTALL: str = "install"

    parser = argparse.ArgumentParser(description=f'')
    parser.add_argument("--dry-run", action='store_true')
    parser.add_argument("--log-level", type=str, choices=['DEBUG', 'INFO', 'ERROR', 'DISABLED'],
                        default='INFO', help='Log level')
    subparsers = parser.add_subparsers(dest='subparser_name')

    # --------------------------------------------------------------------------------------------------------
    # TLS

    parser_tls = subparsers.add_parser(ARG_TLS, help=f'Generate TLS pairs')
    parser_tls.add_argument('--out-dir', required=True, type=str)
    parser_tls.add_argument('--src-root', required=False, type=str, default="~/devel/scripts/tarasovka/thermo_rust")

    # --------------------------------------------------------------------------------------------------------
    # install

    parser_install = subparsers.add_parser(ARG_INSTALL, help=f'Install server or client systemd units')
    parser_install.add_argument('--server', action='store_true', required=False, help='Install server systemd units')
    parser_install.add_argument('--sensor', action='store_true', required=False, help='Install sensor systemd units')



    # -----------------------------------------------------------------------------------------------

    args = parser.parse_args()
    log_levels = {'DEBUG': logging.DEBUG, 'INFO': logging.INFO, 'ERROR': logging.ERROR, 'DISABLED': logging.CRITICAL + 1}
    logger.setLevel(level=log_levels[args.log_level])


    if args.subparser_name == ARG_TLS:
        generate_tls(pl.Path(args.out_dir), pl.Path(args.src_root).expanduser().resolve(), args.dry_run)

    if args.subparser_name == ARG_INSTALL:
        if not (args.server or args.sensor):
            parser_install.error("At least one of --server or --sensor must be specified.")
        if args.server:
            install_server(args.dry_run)
        if args.sensor:
            install_client(args.dry_run)



if __name__ == "__main__":
    main()
