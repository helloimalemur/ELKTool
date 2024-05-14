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
    Tool will create a daily elastic "snapshot" to the configured default snapshot repository.

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
## Alert index for pagerduty
```shell
{
  "title": "{{context.title}}",
  "content": "::PAGERDUTY::{{context.message}}"
}
```

### ```Settings.toml```
Enable or disable backups via ```backups_enabled="true"``` \
Specify remote backup vs local backup via ```remote_copy_enabled="false"``` \

[//]: # (Specify threshold for snapshot ```snapshot_inverval_days="30"```)
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
snapshot_backup_enabled="true" # enable/disable backups entirely
snapshot_repo_backup_drive="/dev/nvme3n1p1" # Elastic drive (not the backup drive, but the drive live elastic data is stored on)
snapshot_min_free_space="1300000" # minimum free space to initiate snapshot creation (1300000 = 1300GB)
#####
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

### config/Transforms.toml
Parses value from source field into a new index field (currently only working for url parameters) \
```toml
[[entry]]
index_prefix = "haproxy-files-2024.05.*"
source_field = "message"
destination_field = "loginId"
transform_type = "url_param"
needle = "loginId="
total_to_process = 6000
```


## Development and Collaboration
#### Feel free to open a pull request, please run the following prior to your submission please!
    echo "Run clippy"; cargo clippy -- -D clippy::all
    echo "Format source code"; cargo fmt -- --check
