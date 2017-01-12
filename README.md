# axfrnotify

Sends a NOTIFY message to a secondary name server to initiate a zone refresh for a specific domain name.

axfrnotify sends a special DNS request called a NOTIFY to inform a secondary DNS server to update its zone information for a specific domain. The mechanism is described in [RFC 1996](https://www.ietf.org/rfc/rfc1996.txt). Once a secondary DNS server receives this request it shall initiate a zone transfer from the primary DNS server responsible for the particular zone; see [RFC 5936](https://tools.ietf.org/html/rfc5936)

RFC 1996 only specifies notification for the SOA resource record type, but axfrnotify allows you to send notification for other resource record types as well.

Since UDP is used and packet loss may occur, axfrnotify has an optional retry mechanism that resends the request in case of no or a unsuccessful responses. I've seen server to deliberately drop the first notification and only successfully answer a second, consecutive request.

## Usage

```plain
USAGE:
    axfrnotify [FLAGS] [OPTIONS] <domain name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Set verbose output

OPTIONS:
    -p, --port <port>                    Set the secondary's port; default is 53
    -t, --type <record_type>             Set the record type to send (A, AAAA, CHAME, MX, NS, PTR, SOA, SRV, TXT, ANY, AXFR); default is SOA
    -r, --retries <retries>              Set the number of retries if notification fails; default is 0
    -s, --secondary <IP or host name>    Set the secondary name server to notify; default is 127.0.0.1

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

* Move repo to trust-dns org.
* Use `From<ClientError>` for exit codes.

---

We stand on the shoulders of giants. Thanks to [bluejekyll](https://github.com/bluejekyll) for [TRust-DNS](http://trust-dns.org) which is the foundation for this little helper.

