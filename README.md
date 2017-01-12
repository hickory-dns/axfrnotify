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
    -t, --type <record_type>             Set the record type to send (A, AAAA, CHAME, MX, NS, PTR, SOA, SRV, TXT, ANY, AXFR); defaults to SOA
    -r, --retries <retries>              Set the number of retries if notification fails; defaults to 0
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

* Use `From<ClientError>` for exit codes.
* Move repo to trust-dns org.

---

We stand on the shoulders of giants. Thanks to [bluejekyll](https://github.com/bluejekyll) for [TRust-DNS](http://trust-dns.org) which is the foundation for this little helper.

