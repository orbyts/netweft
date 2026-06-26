# Verification

```bash
netweft validate
netweft show dns
netweft show proxy
netweft show env --host nexus
```

BIND:

```bash
ssh -t suhail@10.214.90.10 '''sudo docker exec bind9 named-checkconf /etc/bind/named.conf'''
```

Nginx:

```bash
ssh -t suhail@10.214.90.10 '''sudo docker exec nginx-native nginx -t'''
```

Certificate:

```bash
openssl s_client -connect 10.214.90.10:443 -servername dsm.suhail.ink   </dev/null 2>/dev/null | openssl x509 -noout -subject -issuer -dates -ext subjectAltName
```
