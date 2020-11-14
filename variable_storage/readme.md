# Datastorage Format
The database uses a block layout to store variable length data.
## Block Format
```
| is used (u32) | size (u32) |
|<-----next addr (u64)------>|
|<----------Data------------>|
```
## General Layout
```
*--------------*--------*    *--------
| Key Listing  |  Data  |... |  Data
*--------------*--------*    *--------
```
## Key Listing format
key number is the index of BlockNumber/sizeof(u64)
```
| Block Number(u64) |
```
