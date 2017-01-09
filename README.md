# axfrnotify

Sends an NOTIFY message to a secondary name server to initiate a zone refresh for a specific domain name.

## Usage

```plain
USAGE:
    axfrnotify [FLAGS] [OPTIONS] <domain name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Set verbose output

OPTIONS:
    -p, --port <port>                    Set the secondary's port; defaults to 53
    -s, --secondary <IP or host name>    Set the secondary name server to notify; defaults to 127.0.0.1

ARGS:
    <domain name>    Domain name to notify about
```

## Example

```plain
> axfrnotify -v -s 8.8.8.8 example.com
Sending notify for domain 'example.com' to secondary '8.8.8.8:53'.
Successfully sent notification and received positive response.
```

## TODOs

* Allow for notification for different record types -- cf. https://github.com/bluejekyll/trust-dns/blob/master/client/src/rr/record_type.rs#L83.
* Use `From<ClientError>` for exit codes.
* Move repo to trust-dns org.
* Add Travis CI packing for Debian.

---

We stand on the shoulders of giants. Thanks to [bluejekyll](https://github.com/bluejekyll) for [TRust-DNS](http://trust-dns.org) which is the foundation for this this little helper.

