# Boxer  

### Features (so far)

automatic rotation for 3 classes (Warlock, Warrior, Enchanter)  
multi client support  
automatic looting/discarding of loot (including full inventory detection)  
looting all runes  
implemented loot filter  
automatic buffing of party members  
accurate detection of different states  (death, in town, fighting, looting...)  
selection of auto-attack (primary/ranged)  
accurate tracking of cooldowns, cooldown reductions, skill haste (frenzy, augmentation)  
tracking of buffs/debuffs  
automatically using hp pots on low health  

### Tested classes
Enchanter, Warlock, Warrior (atm.)  

### Missing features (may or may not do in future)
auto-explore  
auto town -> dungeon cycle    
auto inventory management (sell stuff in town, store in the bank)  
auto level up  

### TODO (will do)
loot bugs   
auto open inventory  
eventually add support for more classes  

### Notes

Tested with 3 client windows. May or may not perform worse with 5 (max) clients because of racing threads (only one window can be focused at a time). This is a limitation of not being able to send key inputs to unfocused window without dll-injection hacks.  
Character loss of control (stun, silence etc.) will screw a rotation. It's usually a mild inconvenience unless it happens during long buff application. It is hard to detect so I may not bother fixing/detecting this.  