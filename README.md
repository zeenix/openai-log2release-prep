# **openai-log2release-prep**

Training data creator for tuning OpenAI models to generate release notes from git logs.

At the moment, it's a bit specific to zbus and hence the following is assumed:

* Projects is hosted on a Gitlab instance.
* The repo has subcrates/subprojects.
* Tags are named `<subproject>-VERSION`.

## Usage

```sh
export GITLAB_TOKEN="YOUR_TOKEN"
cargo run /home/zeenix/checkout/dbus/zbus gitlab.freedesktop.org/dbus/zbus > /tmp/training_data.json
```

**Note:** At the moment you can't specify the full URL or a trailing slash in the Gitlab path.

Then follow [these steps](https://beta.openai.com/docs/guides/fine-tuning) to train the model.

**Note:** I recommend taking a quick look at the data and remove any obvious noise first.
