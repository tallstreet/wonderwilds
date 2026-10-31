from .common import *
def graph(vals,scale=1,style='bar'):return {'kind':'graph','labels':[f'Day {j+1}' for j in range(len(vals))],'values':vals,'scale':scale,'style':style}
def build():
 for skill in ['MY109','MY209','MY307']:
  for suffix in ['record','read','compare','combine']:
   def make(i):
    scale=2+i%9 if skill=='MY307' else 1;a=(1+i%6)*scale;b=(2+i%5)*scale;c=(3+i%4)*scale
    prompt='How many were counted in category A?' if suffix in ['read','record'] else 'How many were counted altogether?' if suffix=='combine' else 'How many more are in A than B?' if a>=b else 'How many more are in B than A?'
    expected=a if suffix in ['read','record'] else a+b+c if suffix=='combine' else abs(a-b)
    return number(prompt,expected,[{'kind':'graph','labels':['A','B','C'],'values':[a,b,c],'scale':scale,'style':'bar'}],'Read the scale and category labels before calculating.',representation=['picture_graph','tally_table'][i%2] if skill=='MY109' else ['bar_graph','data_table'][i%2] if skill=='MY209' else ['scaled_bar_graph','data_table'][i%2])
   nums(skill,suffix,make)
 for suffix in ['language','likelihood_scale']:
  nums('MY308',suffix,lambda i:choice(f'A bag has {2+i} red counters and no blue counters. Drawing blue is…','impossible',['unlikely','likely','certain'],[text(f'Every counter is red; {2+i} counters are in the bag.')],'An impossible event has no possible outcome.',representation=['probability_scale','event_cards'][i%2]))
 for suffix in ['grid_reference','compass','map_legend']:
  def mapping(i):
   size=4+i%5;x=1+i%(size-1);y=1+(i*3)%(size-1)
   if suffix=='compass':return choice('North points up. Which direction is right?','east',['west','south'],[{'kind':'grid','size':size,'x':x,'y':y}],'Use the compass: north, east, south, west.',representation='compass_rose')
   if suffix=='map_legend':return choice('The map key says ★ means a campsite. What does ★ mark?','campsite',['bridge','school'],[{'kind':'grid','size':size,'x':x,'y':y}],'Read the map key before interpreting symbols.',representation='grid_map')
   return number('What are the marked point’s coordinates?',[x,y],[{'kind':'grid','size':size,'x':x,'y':y}],'Read across first, then up.',representation='grid_map',fields=[{'id':'x','label':'Across','format':'integer'},{'id':'y','label':'Up','format':'integer'}])
  nums('MY309',suffix,mapping)
 for suffix in ['read_line','complete_point','trend_prediction']:
  nums('MY406',suffix,lambda i: number('What is the value at day 3?',i+6,[graph([i+2,i+4,i+6,i+8],2,'line')],'Follow day 3 up to the line, then across to the value axis.',representation=['line_graph','data_table'][i%2]) if suffix!='trend_prediction' else choice('If the pattern continues, what is a reasonable next value?',i+10,[i+8,i+4],[graph([i+2,i+4,i+6,i+8],2,'line')],'This is a prediction from a pattern, not a guaranteed outcome.',representation='line_graph'))
 for skill in ['MY407','MY608']:
  for suffix in (['fraction_probability','compare','experiment'] if skill=='MY407' else ['complement','exclusive','compare']):
   def chance(i):
    total=2+i%(5 if skill=='MY407' else 9);red=1+i%(total-1)
    if suffix=='exclusive':return choice('One counter is drawn. Can it be both red and blue at once?','No',['Yes'],[table(['Colour','Count'],[['Red',red],['Blue',total-red]])],'A single counter has one colour, so these events are mutually exclusive.',representation='venn_diagram')
    if suffix=='compare':return choice('Which colour is more likely?','Red' if red>total-red else 'Blue' if red<total-red else 'Equal chance',['Red','Blue','Equal chance'],[table(['Colour','Count'],[['Red',red],['Blue',total-red]])],'Compare favourable outcomes in the same sample space.',representation='probability_scale')
    if suffix=='experiment':return number('These are recorded trial results. How many red draws were observed?',red,[table(['Observed colour','Number of trials'],[['Red',red],['Blue',total-red]])],'Observed results can differ from theoretical predictions.',representation='spinner_model')
    return number('What is the probability of blue?' if suffix=='complement' else 'What is the probability of red?',f'{total-red if suffix=="complement" else red}/{total}',[table(['Colour','Count'],[['Red',red],['Blue',total-red]])],'Favourable outcomes divided by all equally likely outcomes.',representation='probability_scale',format='fraction')
   nums(skill,suffix,chance,24)
 for suffix in ['read_point','plot_point','value_table','relationship']:
  nums('MY506',suffix,lambda i:number('Read the marked point’s x and y coordinates.',[i%11,(i*3)%11],[{'kind':'grid','size':10,'x':i%11,'y':i*3%11}],'The x coordinate comes first; the y coordinate comes second.',representation=['cartesian_grid','table_of_values'][i%2],fields=[{'id':'x','label':'x coordinate','format':'integer'},{'id':'y','label':'y coordinate','format':'integer'}]),22)
 for suffix in ['mean','median','mode','range','choose_measure']:
  def statistics(i):
   values=[i+1,i+3,i+3,i+5,i+8];expected={'mean':Fraction(sum(values),len(values)),'median':i+3,'mode':i+3,'range':7}
   if suffix=='choose_measure':return choice('One very large outlier is added. Which measure best describes a typical value?','median',['mean','range'],[table(['Values'],[[v] for v in values+[i+100]])],'The median is less affected by an extreme outlier.',representation='dot_plot')
   return number(f'Find the {suffix}.',str(expected[suffix]),[table(['Values'],[[v] for v in values[::-1]])],'Sort values for the median; add and divide for the mean; count repeats for mode; subtract extremes for range.',representation=['data_table','dot_plot'][i%2],format='fraction' if suffix=='mean' else 'integer')
  nums('MY507',suffix,statistics)
 for suffix in ['distribution','compare_graphs','misleading_graph']:
  nums('MY607',suffix,lambda i:number('What is the difference between the largest and smallest values?',7,[table(['Values'],[[i+v] for v in [1,3,3,5,8,4,6,7]])],'Subtract the minimum from the maximum.',representation='stem_leaf_plot') if suffix=='distribution' else choice('Which change makes a bar graph misleading?','Starting the value axis above zero without making that clear',['Showing the same data in a table','Labelling the categories'],[graph([i+3,i+4,i+5],1)],'A truncated baseline can exaggerate differences.',representation='misleading_graph'))
