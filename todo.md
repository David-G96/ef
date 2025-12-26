# ef

Sort your items

## TODO

- [ ] 使用epoch增强异步回调报告的准确度。每个model实例都必须携带epoch信息，同时发送task时携带id，这样即使是不同model发送了同id的task也可以分辨。epoch信息应该是非侵入式的，由运行时管理；id应该是属于model内部的。Epoch可以应用在Cmd和Msg上。一旦当前model的epoch和Msg epoch不相符，那么它就会立即被丢弃。
  - [ ] Sub-task or description  

## Completed Column ✓

- [x] Completed task title  
