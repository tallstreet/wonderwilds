from .common import *
def build():
 nums('M01','one_for_each',lambda i:assign('Give one gift to each friend.',[f'Gift {j+1}' for j in range(1+i%6)],[f'Friend {j+1}' for j in range(1+i%6)],representation=['object_pairing','recipient_slots'][i%2]),6)
 nums('M01','repair_pairing',lambda i:assign('Make sure every friend has exactly one gift.',[f'Parcel {j+1}' for j in range(2+i%5)],[f'Place {j+1}' for j in range(2+i%5)],representation='recipient_slots'),5)
 for suffix,layout in [('tiny_groups','rows'),('match_patterns','scattered')]:
  nums('M02',suffix,lambda i:choice('How many dots do you see?',1+i%4,[1+(i+1)%4,1+(i+2)%4],[coll([1+i%4],['Dots'],layout)],'Look for pairs or a whole small group.',representation=['brief_dot_pattern','structured_object_pattern'][i%2]),4)
 for suffix,layout in [('scattered','scattered'),('ten_frames','ten_frame'),('numeral_match','rows')]:
  nums('M03',suffix,lambda i:number('How many objects are there?',i+1,[coll([i+1],['Objects'],layout)],'Count each object once. The last number is the total.',representation='ten_frame' if layout=='ten_frame' else 'movable_objects'),20)
 for suffix in ['more','fewer','same']:
  def compare(i,mode=suffix):
   a=1+i%10;b=a if mode=='same' and i%2==0 else 1+(i*3+2)%10
   good=('Same amount' if a==b else ('Group A' if (a>b)==(mode!='fewer') else 'Group B')) if mode!='same' else ('Yes' if a==b else 'No')
   return choice('Are the groups the same amount?' if mode=='same' else f'Which group has {mode}?',good,['No' if good=='Yes' else 'Yes'] if mode=='same' else ['Group A','Group B','Same amount'],[coll([a,b],['Group A','Group B'],'rows' if i%2 else 'scattered')], 'Compare the counts, not how far apart the objects are.',representation=['paired_collections','aligned_rows'][i%2])
  nums('M04',suffix,compare,20)
 nums('M04','order',lambda i:order('Put the collection counts from fewest to most.',[1+i%3,4+i%3,7+i%4],representation='aligned_rows'))
 for suffix,step in [('one_more',1),('one_less',-1)]:
  nums('M05',suffix,lambda i:choice(f'What number is {suffix.replace("_"," ")} than {i+2 if step<0 else i+1}?',(i+2 if step<0 else i+1)+step,[(i+2 if step<0 else i+1)-step],[text('Count forward one step.' if step>0 else 'Count back one step.')], 'Move one step in the direction named.',representation='number_path'),18)
 nums('M05','missing_number',lambda i:number('Which number comes next?',i+4 if i%2==0 else 17-i,[text(f'{i+1}, {i+2}, {i+3}, ?' if i%2==0 else f'{20-i}, {19-i}, {18-i}, ?')], 'Keep counting in the same direction.',representation='countable_flower_sequence'),16)
 for suffix in ['missing_part','two_parts','split_collection']:
  nums('M06',suffix,lambda i:number('How many more make the whole?',(2+i%9)-(1+i%(1+i%9)),[text(f'Whole: {2+i%9}. One part: {1+i%(1+i%9)}.')], 'Subtract the known part from the whole.',representation=['countable_planks','part_whole_objects'][i%2]))
 for suffix,joining in [('join',True),('take_away',False)]:
  nums('M07',suffix,lambda i:number('How many are there now?' if joining else 'How many are left?',(1+i%5)+(1+i%5) if joining else (6+i%5)-(1+i%5),[text(f'{1+i%5} + {1+i%5} = ?' if joining else f'{6+i%5} − {1+i%5} = ?'),coll([1+i%5,1+i%5] if joining else [6+i%5],layout='rows')],'Combine the groups.' if joining else 'Take away the stated amount.',representation=['movable_crystals','change_story_objects'][i%2]))
 nums('M07','picture_equation',lambda i:choice('Which equation matches the two groups?',f'{1+i%5} + {1+i%5} = {2*(1+i%5)}',[f'{1+i%5} − {1+i%5} = 0',f'{1+i%5} + 1 = {2+i%5}'],[coll([1+i%5,1+i%5])],representation='change_story_objects'))
 nums('M08','share',lambda i:assign(f'Share {2*(1+i%6)} berries equally between two plates.',['Berry']*(2*(1+i%6)),['Plate A','Plate B'],capacity=6,representation='plates_and_objects'),6)
 nums('M08','per_recipient',lambda i:number(f'Share {2*(1+i%6)} berries equally between two friends. How many each?',1+i%6,[coll([2*(1+i%6)])],'Give one to each friend in turn.',representation='recipient_slots'))
 nums('M08','fair_shares',lambda i:choice('Are these shares fair?','Yes' if i%2 else 'No',['No' if i%2 else 'Yes'],[coll([2+i%4,2+i%4 if i%2 else 1+i%4],['Plate A','Plate B'])], 'Fair shares have the same amount.',representation='plates_and_objects'))
 patterns=[['circle','triangle'],['circle','circle','triangle'],['circle','triangle','triangle'],['star','square'],['star','star','square'],['star','square','square']]
 for suffix in ['continue','repeating_unit','build_pattern']:
  nums('M09',suffix,lambda i:order('Build two repeats of this pattern.',patterns[i%6]*2,[text('Repeating unit: '+' → '.join(patterns[i%6]))], 'Repeat the whole unit, keeping its order.',representation=['object_pattern','shape_pattern'][i%2],pictures=True),6)
 for suffix in ['name_shape','features','silhouettes']:
  nums('M10',suffix,lambda i:choice('What is the name of this shape?',['triangle','square','circle'][i%3],['triangle','square','circle'],[pic(['triangle','square','circle'][i%3])],'Look at the sides and corners.',representation=['shape_tiles','object_silhouettes'][i%2]),6)
 for suffix in ['position','movement']:
  nums('M11',suffix,lambda i:choice('Where is the star relative to the circle?' if suffix=='position' else 'Which way should you move?',['above','below','beside'][i%3],['above','below','beside'],[text(['The star is directly higher than the circle.','The star is directly lower than the circle.','The star is next to the circle.'][i%3])], 'Use the circle as your starting reference.',representation=['object_scene','grid_path'][i%2]),6)
 for suffix,unit in [('length','cm'),('mass','grams'),('capacity','cups')]:
  nums('M12',suffix,lambda i:choice(f'Which has greater {suffix}?','A',['B','Same'],[table(['Object',unit],[['A',i+3],['B',i+1]])],'Compare using the same unit.',representation='aligned_objects' if suffix=='length' else 'balance_or_container'))
 events=[['Wake up','Eat breakfast','Go to school'],['Put on socks','Put on shoes','Walk outside'],['Mix the batter','Bake the cake','Eat the cake'],['Plant seed','Water seed','See sprout'],['Wash hands','Prepare food','Eat food'],['Pack bag','Leave home','Arrive at school']]
 for suffix in ['order_events','before_after','day_stages']:
  nums('M13',suffix,lambda i:order('Put these events in order.',events[i%6],hint='Think about what must happen before the next event.',representation=['event_cards','day_sequence'][i%2]),6)
 for suffix in ['sort_attribute','belongs']:
  nums('M14',suffix,lambda i:assign('Sort the shapes by their number of sides.',['Triangle','Square','Triangle','Square'],['3 sides','4 sides'],[0,1,0,1],capacity=4,representation=['object_sort','attribute_cards'][i%2]),6)
 for suffix in ['record_counts','read_graph','compare_categories']:
  nums('M15',suffix,lambda i:number('How many votes did apples get?',1+i%10,[{'kind':'graph','labels':['Apples','Pears'],'values':[1+i%10,1+(i*3)%10],'scale':1,'style':'bar'}],'Read only the apple category.',representation=['object_graph','picture_graph'][i%2]))
 for suffix in ['count_tens_ones','build_number','regroup']:
  nums('M16',suffix,lambda i:number('What number is shown?',10+i,[{'kind':'place_value','value':str(10+i),'places':2}], 'A ten is ten ones.',representation=['bundled_seeds','tens_and_ones'][i%2]),21)
 for suffix in ['equal_groups','skip_count','grouped_total']:
  nums('M17',suffix,lambda i:number(f'{2+i%4} rows have {2+i//4%3} seeds each. How many seeds?',(2+i%4)*(2+i//4%3),[text(f'{2+i%4} equal groups of {2+i//4%3}')], 'Add one equal group at a time.',representation=['planting_rows','grouped_objects'][i%2]))
 for suffix in ['equal_parts','halves_quarters','partition']:
  nums('M18',suffix,lambda i:choice('What fraction is one equal part?',f'1/{2+i%3}',[f'1/{3+i%3}','1'],[{'kind':'fraction','numerator':1,'denominator':2+i%3}],'The denominator counts all equal parts.',representation=['partitioned_shape','folded_whole'][i%2]),6)
 for suffix in ['forward','backward','missing_terms']:
  nums('MY101',suffix,lambda i:number('Which number comes next?',(i%6)*10+(3 if suffix!='backward' else 0)*[1,2,5,10][i%4],[text(', '.join(str((i%6)*10+(j if suffix!='backward' else 3-j)*[1,2,5,10][i%4]) for j in range(3))+', ?')],'Keep the same step size and direction.',representation=['number_line','hundreds_chart'][i%2]),24)
 for skill,maxn,places in [('MY102',99,2),('MY201',999,3),('MY301',9999,4),('MY401',1000000,7)]:
  vals=[10,21,40,56,78,99,12,34,60,81,90,97] if places==2 else [10**(places-1) if places<7 else 1000,120,304,560,708,999,100,202,450,801,900,997] if places==3 else [100,1000,1020,2304,4050,5607,7080,8102,9000,9999,6789,1234] if places==4 else [1000,10000,100000,1000000,102030,230405,405060,560708,708090,810203,900000,999999]
  for suffix in ['read_write','digit_value','expand']:
   nums(skill,suffix,lambda i:number('Write the number shown.' if suffix=='read_write' else 'What is the value of the tens digit?',vals[i] if suffix=='read_write' else (vals[i]//10%10)*10,[{'kind':'place_value','value':str(vals[i]),'places':places}], 'Each place is ten times the place on its right.',representation='base_ten_blocks' if skill in ['MY102','MY201'] and i%2 else 'place_value_chart'))
  if skill=='MY301':
   for unit in [10,100]:nums(skill,f'round_{unit}',lambda i:number(f'Round {vals[i]} to the nearest {unit}.',((vals[i]+unit//2)//unit)*unit,[text(str(vals[i]))],'Find the two neighbouring multiples; choose the nearer one.',representation='number_line'))
  if skill=='MY401':nums(skill,'compare_order',lambda i:order('Order from least to greatest.',sorted([vals[i],vals[(i+1)%12],vals[(i+2)%12]]),representation='number_expander'))
 nums('MY103','symbols',lambda i:choice('Which symbol belongs between these numbers?','=' if i%3==0 else '<' if i%3==1 else '>',['<','=','>'],[text(f'{20+i} ? {20+i if i%3==0 else 23+i if i%3==1 else 17+i}')],'Compare tens before ones.',representation='place_value_chart'))
 nums('MY103','greatest_least',lambda i:choice('Which number is greatest?',40+i,[20+i,30+i],[text('Compare the tens, then the ones.')],representation='number_line'))
 nums('MY103','order',lambda i:order('Order from least to greatest.',[10+i,30+i,50+i],representation='number_line'))
 nums('MY108','repeating_core',lambda i:choice('Which repeating unit makes the pattern?', ['circle triangle','circle circle triangle','circle triangle triangle'][i%3],['circle','triangle'],[text(' '.join(patterns[i%3]*3))],'Find the shortest block that repeats.',representation='pattern_tiles'),6)
 nums('MY108','number_pattern',lambda i:number('Which number comes next?',i+3*(2+i%4),[text(', '.join(str(i+j*(2+i%4)) for j in range(3))+', ?')],'Find the constant amount added each time.',representation='number_sequence'))
 nums('MY108','create_pattern',lambda i:order('Build an increasing number pattern with equal jumps.',[i,i+2,i+4,i+6],representation='number_sequence'))
