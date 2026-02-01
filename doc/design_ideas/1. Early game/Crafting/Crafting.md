# Crafting

## Property types
* items
* modules
* vessels

## Property complexity types
* Basic - Metals, Liquids, Powders and other basic materials
    - Can be influenced by "Efficiency" modifier while production
    - Complex modifiers can not be put on it
* Complex - Usually some complex maschinery
    - Complex modifiers can be put on it

## Modifiers
All material types of property can have modifiers that affact final product properties.

If one property is created from some set of other properties then modifiers are passed from input to final product during crafting process. If more then one input has modifiers, the final modifier is the sum. For example if you build engine from some items where one item has modifier "Power + 1%", another one "Power + 2%" and all the others does not have any modifiers, then final engine "Power + 3%"

If a property have modifiers that does not match its purpose, then those specific modifiers are ignored. For example if engine have modifier "Accuracy" it is ignored since engines don't have accuracy.

Modifier can be basic or complex compatible. While basic compatible modifier can be put on any property, complex only on complex maschinery.

### List of modifiers
* Power `Physics science points`
    - Engines = Increases thrust
    - Radar = Can detect fainter targets
    - Explosives = More power
    - Firearm = Increases projectile speed
* Efficiency `Enginering science points`
    - Engines = Increases specific impulse
    - Crafting module (Only when producing basic materials) = uses less resources
* Heat Resistance (Basic) `Physics + Material science points`
    - Any module = Can withstand more heat
* Durability (Basic) `Material science points`
    - Modules that have a lifetime = the lifetime is increased
* Strength (Basic) `Material science points`
    - Any module = More health
* Luxurity (Basic) `Any science points`
    - Any module = More atractable for a person that has high need of laxurity
* Precision `Computation science points`
    - Weapos = Increases accuracy
    - Crafting modules = Increases product modifier limit
    - Radars = Higher resolution
* Speed `Engeneering + Computation science points`
    - Turrets = Increases rotation speed
    - Coil guns / Rail guns = Increases projectile speed
    - Crafting modules = Increases crafting speed
    - Radars = Increases turn / update rate
    - Weapons with high fire rate = Increases fire rate
* Comfortability (Basic) `Material + Biology science points`
    - Any module that person can operate on = Persons stamina decreases slower
* Quality
    - Any module = Influences failure chance. Failure chance = 100% - Quality. Usually 100% but can be less then 100% if produced by unqualified person, crafting module with Quality < 100% or in science experiment.
* X Modifier Changing
	- Crafting modules = Cnages modifier X of output (On some items involved in crafting the crafting modules change is applied recursively.)
 
### Recursive Modifier Application

Example: 

There is ware -> ware crafting module "Fabricator". Its recipe is `100 Steel + 20 Plastic + 10 Microelectronic + 1 Robo-Toolkit -> Fabricator`

In this example "X Modifier Changing" Modifier can be put only on Robo-Toolkit and Fabricator. For example the exact modifier is 
"Heat Resistance Modifier Changing +10%". If the fabritator has this modifier, any output will recieve "Heat Resistance +10%" modifier, and if output is "Robo-Toolkit", it will recieve "Heat Resistance Modifier Changing +10%" modifier in addition to "Heat Resistance +10%", so if player has fabricator with some modifier changing modifier they can clone it. But first "Robo-Toolkit" with this specific modifier must be obtained somewhere else: In science lab.

### Science lab

There is a module "Science lab" that can produce items like "Fabricator" but uses science points and has a chance of "Failed experiment" which adds negative modifiers to the product or adds "Defect" modifier.

There are several types of science points:
* Material science - Used in experiments involving modiefiers such as "Durability", "Strength", "Heat Resistance", "Luxurity"
* Computation science - Experiments involving "Precision",  "Luxurity" 
* Biology - Experiments involving "Comfortability", "Luxurity"
* Phisics - Experiments involving "Power", "Luxurity"
* Engeneering - Experiments involving "Efficiency", "Speed", "Luxurity"

## Ware -> Ware Crafting
Done through a puzzle minigame that must be interesting

## Ware -> Modules Crafting
Done through a minigame that must be interesting

## Item + Ware -> Ware Crafting

## Item + Module -> Module Crafting

Example:
LFO Engine:
	Reaction Chamber
	Nozzle
    Injector
    Ignitor
    Tubes
    




