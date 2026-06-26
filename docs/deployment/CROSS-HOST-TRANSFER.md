# Cross-host transfer

Create staging:

```bash
ssh suhail@10.214.90.10 '''
rm -rf /tmp/netweft-artifact-staged
mkdir -p /tmp/netweft-artifact-staged
'''
```

Copy directory contents:

```bash
scp -r "/local/generated/path/."   suhail@10.214.90.10:/tmp/netweft-artifact-staged/
```

Use `/.` to copy contents without adding another directory level.

Variables defined in one shell are not available in later independent commands. Prefer literal paths or self-contained blocks.
