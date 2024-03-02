# elktool

## Setup
extracts and builds within ```/var/lib/elktool/``` \
creates systemd service ```elktool```

## install manually
```shell
bash -e install.sh
```

## About
#### Manage Elastic Index lifetimes without the complexity
    Simply set a maximum lifetime.
    The Tool closes and deletes indices matching policies in the ```Policy.toml``` file.

### Create backups
    Configure the options in the ```Settings.toml``` file.
    Tool will create an elastic "snapshot", archive the snapshot into a tarball.
    Then scp or locally copy it to the specified destination.

### Alert index
    Create Rules in Kibana to create an index named "alert-index"
    The Tool checks for an index named "alert-index" containing alert information.
    An alert summary is then sent to the configured discord webook and email recipients.

##### Alert index connector format
```shell
{
"title": "{{context.title}}",
"content": "{{context.message}}"
}
```

### ```Settings.toml```
Enable or disable backups via ```backups_enabled="true"``` \
Specify remote backup vs local backup via ```remote_copy_enabled="false"``` \
Specify threshold for snapshot ```snapshot_inverval_days="30"```
Specify threshold for backup ```backup_inverval_days="34"``` # longer than snapshot recommended
Specify loop delay via ```delay="43200"```
Specify whether to run Lifetime Management and Backup procedure on start ```run_lm_on_start="false"```
Specify whether to enable alerting ```alerting_enabled="true"```

### config/Settings.toml
```toml
delay="43200" # delay on loop
# elastic url and credentials
elastic_url="https://yourelkinstance:9200"
elastic_user=""
elastic_pass=""
run_lm_on_start="true"
alerting_enabled="true"
# discord server webook
discord_webhook_url=""
# snapshot settings
snapshot_repo_name="backup_snapshot_repo" # name of elastic snapshot repo - DO NOT CHANGE
snapshot_repo_path="/mnt/backup_drive/backup_snapshot_repo/" # elastic repo path - update elasticsearch.yml
snapshot_last_timestamp="/mnt/backup_drive/last_snapshot" # timestamp of last snapshot
backup_last_timestamp="/mnt/backup_drive/last_backup" # timestamp of last backup
snapshot_backup_enabled="true" # enable/disable backups entirely
snapshot_repo_backup_drive="/dev/nvme3n1p1" # Elastic drive (not the backup drive, but the drive live elastic data is stored on)
snapshot_min_free_space="1300000" # minimum free space to initiate snapshot creation (1300000 = 1300GB)
## Backup server settings
remote_copy_enabled="false" # true to copy over ssh using settings below, false to copy locally to the same destination path
backups_enabled="true" # enable or disable copying snapshot to backup and archiving
ssh_from_host="192.168.0.162" ### SOURCE HOST if service is remote to elastic
backup_server_host="127.0.0.1" # SSH remote host
backup_server_ssh_user="root" # SSH remote user
backup_server_ssh_port="8822" # SSH remote port
backup_server_ssh_key="" # SSH remote authorized public key
backup_server_src_dir="/mnt/backup_drive/backup_snapshot_repo/"
#backup_server_src_dir="/tmp/test/"
#backup_server_dest_dir="/mnt/backup_drive/backup_snapshot_repo/" # remote
backup_server_dest_dir="/mnt/backup_drive/raw/" # destination directory for raw data pre-compression (destination for remote AND local)
backup_server_archive_dest_dir="/mnt/backup_drive/archive/" # destination directory for post-compression (destination for remote AND local)
## interval settings
snapshot_inverval_days="30"
backup_inverval_days="34" ### Advised to keep several day lag from snapshot backup, to allow time for the snapshot to finish
max_async_search_response_size="20MB"
### SMTP settings
smtp_enabled = "false"
smtp_host = "smtp-relay.gmail.com"
smtp_port = "587"
smtp_require_auth = "true"
smtp_username = ""
smtp_password = ""
smtp_recipient_1 = ""
smtp_recipient_2 = ""
smtp_recipient_3 = ""
smtp_recipient_4 = ""
smtp_recipient_5 = ""
smtp_recipient_6 = ""
smtp_recipient_7 = ""
```


### config/Policy.toml
Matches prefix for policy and index name for matching on indexes, delimited by '_'. \
```delete_``` and ```close_``` are available. \
```policy_index="days"```
```toml
# HAPROXY
close_haproxy="45"
delete_haproxy="90"
```


## Development and Collaboration
#### Feel free to open a pull request, please run the following prior to your submission please!
    echo "Run clippy"; cargo clippy -- -D clippy::all
    echo "Format source code"; cargo fmt -- --check
