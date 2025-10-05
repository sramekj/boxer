### Tested classes
Enchanter, Warlock, Warrior (atm.)  

### Missing features (may or may not do in future)
auto-explore  
auto town -> dungeon cycle    
auto inventory management (sell stuff in town, store in the bank)  
auto level up  

### TODO (will do)
loot bugs   
automatic HP pots on low HP  
eventually add support for more classes  

### Notes

Tested with 3 client windows. May or may not perform worse with 5 (max) clients because of racing threads (only one window can be focused at a time). This is a limitation of not being able to send key inputs to unfocused window without dll-injection hacks.  
