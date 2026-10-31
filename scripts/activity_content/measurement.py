from .common import *
def build():
 for skill in ['MY106','MY208']:
  for suffix in (['name_shape','features','sort'] if skill=='MY106' else ['classify','symmetry','transform']):
   def make(i):
    if suffix in ['name_shape','classify']:return choice('Which solid has six square faces?','cube',['sphere','cone'],[text('A solid with 6 equal square faces, 12 edges and 8 vertices.')],'Think about the faces and corners of a dice.',representation='physical_object_sort' if skill=='MY106' else 'shape_image')
    if suffix=='symmetry':return choice('How many lines of symmetry does a square have?',4,[1,2],[pic('square')],'Try a vertical fold, a horizontal fold and diagonal folds.',representation='symmetry_fold')
    if suffix=='transform':return choice('A triangle is turned without changing its size. What happened?','rotation',['reflection','enlargement'],[pic('triangle')],'A turn is a rotation; a flip is a reflection.',representation='shape_image')
    if suffix=='sort':return assign('Sort these objects.',['Square','Cube','Circle','Sphere'],['Flat shapes','Solids'],[0,1,0,1],capacity=4,representation='physical_object_sort')
    return number('How many sides does this shape have?',[3,4,0][i%3],[pic(['triangle','square','circle'][i%3])],'Count straight edges around the shape.',representation='shape_image')
   nums(skill,suffix,make,6)
 for skill in ['MY107','MY206']:
  for suffix in ['length','mass','capacity','area'] if skill=='MY206' else ['length','mass','capacity','equal_units']:
   def make(i):
    if suffix=='length':return number('How long is ribbon A?',i+2,[{'kind':'measure','labels':['Ribbon A'],'lengths':[i+2],'unit':'cm'}],'Read from the starting mark to the ending mark.',representation='ruler_tool' if skill=='MY206' else 'non_standard_unit_measure',unit='cm')
    if suffix=='area':return number(f'A rectangle is {2+i%5} units by {3+i%4} units. What is its area?',(2+i%5)*(3+i%4),[text('Cover the rectangle with equal unit squares.')],'Multiply the number of rows by squares in each row.',representation='ruler_tool',unit='square units')
    unit='grams' if suffix=='mass' else 'cups' if suffix=='capacity' else 'equal blocks'
    return choice(f'Which has more {unit}?','A',['B','Same'],[table(['Object',unit],[['A',i+3],['B',i+1]])],'Use the same unit when comparing.',representation='balance_scale' if skill=='MY206' else 'comparison_image')
   nums(skill,suffix,make)
 for skill,minutes in [('MY110',[0,30]),('MY207',[15,45])]:
  for suffix in ['read_clock','match_digital','set_time']+(['am_pm'] if skill=='MY207' else []):
   def make(i):
    hour=1+i%12;minute=minutes[i%2]
    if suffix=='am_pm':return choice('Is 7 in the morning AM or PM?','AM',['PM'],[text('Pip eats breakfast in the morning before school.')],'AM runs from midnight to before noon.',representation='digital_clock')
    return choice('What time does the clock show?' if suffix!='set_time' else f'Which digital time matches {hour}:{minute:02}?',f'{hour}:{minute:02}',[f'{hour}:{(minute+15)%60:02}',f'{hour%12+1}:{minute:02}'],[{'kind':'clock','hour':hour,'minute':minute}],'The short hand shows the hour; the long hand shows minutes.',representation=['analogue_clock','digital_clock'][i%2])
   nums(skill,suffix,make,24)
 for suffix in ['area','perimeter']:
  nums('MY306',suffix,lambda i:number(f'Find the {suffix} of this rectangle.',(2+i)*(3+i%5) if suffix=='area' else 2*(2+i+3+i%5),[{'kind':'measure','labels':['Length','Width'],'lengths':[2+i,3+i%5],'unit':'cm'}],'Multiply length by width for area.' if suffix=='area' else 'Add all four outside edges.',representation=['grid_model','ruler_tool'][i%2],unit='cm²' if suffix=='area' else 'cm'))
 for suffix in ['composite_area','composite_perimeter','decompose']:
  def composite(i):
   a=5+i;b=4+i%4;cut=2
   if suffix=='decompose':return choice('How can you find the area of an L-shaped garden?','Split it into two non-overlapping rectangles',['Multiply its outer width by height without subtracting the missing part','Add all outside edges'],[text(f'Outer rectangle: {a} by {b}; a 2 by 2 corner is missing.')],'Add non-overlapping parts, or subtract a missing corner.',representation='composite_shape_diagram')
   return number(f'Find the {"area" if suffix=="composite_area" else "perimeter"} of the L shape.',a*b-4 if suffix=='composite_area' else 2*(a+b),[text(f'Outer rectangle {a} cm by {b} cm; remove a 2 cm by 2 cm square from its top-right corner.')],'Subtract the missing area.' if suffix=='composite_area' else 'Trace every outside edge, including the notch.',representation=['grid_model','composite_shape_diagram'][i%2],unit='cm²' if suffix=='composite_area' else 'cm')
  nums('MY405',suffix,composite)
 for suffix in ['elapsed','timetable','convert']:
  def time(i):
   start=8*60+i*7;duration=5+i*10;end=start+duration
   if suffix=='convert':return number(f'How many minutes are in {1+i%2} hours?',60*(1+i%2),hint='Each hour contains 60 minutes.',representation='timeline',unit='minutes')
   return number('How many minutes does the journey take?',duration,[table(['Departure','Arrival'],[[f'{start//60}:{start%60:02}',f'{end//60}:{end%60:02}']])],'Count forward from departure to arrival.',representation=['timeline','digital_clock'][i%2],unit='minutes')
  nums('MY408',suffix,time)
 for suffix in ['unit_cubes','prism_volume','capacity']:
  nums('MY508',suffix,lambda i:number('Find the volume of the rectangular prism.',(1+i%10)*(1+i*3%10)*(1+i*7%10),[table(['Length','Width','Height'],[[1+i%10,1+i*3%10,1+i*7%10]]),text('All dimensions are centimetres. One cubic centimetre holds one millilitre.')],'Multiply length × width × height.',representation=['3d_model','grid_model'][i%2],unit='mL' if suffix=='capacity' else 'cm³'))
 for suffix in ['straight_line','triangle_angle','classify']:
  def angles(i):
   a=20+i*5;b=30+i*3
   if suffix=='classify':return choice(f'A triangle has angles {a}°, {b}° and {180-a-b}°. How is it classified by angle?','right' if 90 in [a,b,180-a-b] else 'obtuse' if max(a,b,180-a-b)>90 else 'acute',['acute','right','obtuse'],[{'kind':'angle','values':[a,b,180-a-b],'triangle':True}],'An acute triangle has all angles below 90°. A right triangle has a 90° angle.',representation='angle_diagram')
   return number('Find the missing angle.',180-a if suffix=='straight_line' else 180-a-b,[{'kind':'angle','values':[a] if suffix=='straight_line' else [a,b],'triangle':suffix!='straight_line'}],'Angles on a straight line and angles inside a triangle each total 180°.',representation=['protractor_tool','angle_diagram'][i%2],unit='degrees')
  nums('MY606',suffix,angles)
