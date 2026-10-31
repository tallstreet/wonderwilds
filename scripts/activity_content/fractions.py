from .common import *
def fraction(n,d):return {'kind':'fraction','numerator':n,'denominator':d}
def build():
 for suffix in ['shape_half','shape_quarter','collection']:
  nums('MY205',suffix,lambda i:number(f'What is one {"half" if i%2==0 else "quarter"} of {4*(i+1)}?',(4*(i+1))//(2 if i%2==0 else 4),[fraction(1,2 if i%2==0 else 4)],'Split the collection into the stated number of equal parts.',representation=['fraction_model','area_model'][i%2]))
 for suffix in ['unit_fraction','place_on_line','compare','order']:
  def make(i):
   d=2+i%7;n=1+i%(d-1)
   if suffix=='order':return order('Order these fractions from least to greatest.',[f'{j}/{d}' for j in range(1,min(d,4))],representation='fraction_model') if d>=3 else choice('Which is greater?','1/2',['0/2'],[fraction(1,2)],representation='fraction_model')
   if suffix=='compare':return choice('Which fraction is greater?',f'{n+1}/{d}',[f'{n}/{d}'],hint='With equal denominators, compare the numerators.',representation='fraction_model')
   return number('What fraction is shaded?' if suffix=='unit_fraction' else 'What fraction names the marked part?',f'1/{d}',[fraction(1,d)],'One equal part has numerator one.',representation='fraction_model' if suffix=='unit_fraction' else 'number_line',format='fraction')
  nums('MY305',suffix,make,14)
 for suffix in ['equivalent','missing_part','locate']:
  nums('MY403',suffix,lambda i:number(f'Complete the equivalent fraction: 1/{2+i%5} = ?/{2*(2+i%5)}',2,[fraction(1,2+i%5)],'Multiply numerator and denominator by the same number.',representation=['fraction_wall','number_line'][i%2]))
 for suffix in ['compare','order','mixed_number','locate']:
  def make(i):
   d=2+i%6;n=d+1+i%d
   if suffix=='mixed_number':return number(f'Write {n}/{d} as a mixed number.',[n//d,n%d,d], [fraction(n,d)], 'Divide the numerator by the denominator; the remainder is the fractional part.',representation='fraction_wall')
   if suffix=='order':return order('Order from least to greatest.',['1/4','1/2','3/4'],[text(f'Use a whole split into {4*(1+i%3)} equal pieces to compare.')],representation='number_line')
   return choice('Which fraction is greater?','1/2',['1/3'],[text(f'Compare one half and one third of the same whole. Example whole has {6*(i+1)} parts.')],'Compare equal wholes, not just the denominators.',representation=['fraction_wall','number_line'][i%2])
  nums('MY502',suffix,make)
 for suffix in ['add','subtract','multiply']:
  def make(i):
   d=2+i%5;n=1+i%d
   result=Fraction(n,d)+Fraction(1,d*2) if suffix=='add' else Fraction(n,d)-Fraction(1,d*2) if suffix=='subtract' else Fraction(n,d)*(2+i%3)
   expression=f'{n}/{d} {"+" if suffix=="add" else "−"} 1/{d*2}' if suffix!='multiply' else f'{n}/{d} × {2+i%3}'
   return number(expression+' = ?',str(result),[fraction(n,d)],'Use equal-sized fractional parts; multiply both parts of an equivalent fraction.',representation=['fraction_model','number_line'][i%2],format='fraction')
  nums('MY503',suffix,make)
 for skill,precision in [('MY404',2),('MY504',3)]:
  for suffix in (['read_model','digit_value','fraction_decimal'] if precision==2 else ['read_order','add','subtract']):
   def make(i):
    scale=10**precision;a=125+i*37 if precision==3 else 5+i*13;b=25+i*3
    value=a+b if suffix=='add' else a-b if suffix=='subtract' else a
    value=min(value,9*scale)
    exact=f'{value//scale}.{value%scale:0{precision}d}'
    stimulus=text(f'{a//scale}.{a%scale:0{precision}d} {"+" if suffix=="add" else "−"} {b//scale}.{b%scale:0{precision}d}') if suffix in ['add','subtract'] else {'kind':'place_value','value':exact,'places':precision+1}
    if suffix=='fraction_decimal':stimulus=text(f'Write {value}/{scale} as a decimal.')
    return number('Write the decimal value.',exact,[stimulus],'Line up equal place values; zeroes may fill empty places.',representation='hundredths_grid' if precision==2 and i%2 else 'place_value_chart',format='decimal')
   nums(skill,suffix,make)
 for suffix in ['hundred_model','convert','application']:
  def percent(i):
   p=5+5*i
   if suffix=='convert':return number(f'Write {p}% as a decimal.',f'{p//100}.{p%100:02}',[text(f'{p} out of 100')],'Percent means per hundred.',representation='number_line',format='decimal')
   return number('What percent is shaded?' if suffix=='hundred_model' else f'What is {p}% of 200?',p if suffix=='hundred_model' else 2*p,[{'kind':'graph','labels':['Shaded','Not shaded'],'values':[p,100-p],'scale':10,'style':'bar'}],'Percent means per hundred.',representation='hundredths_grid')
  nums('MY505',suffix,percent)
 for suffix in ['fraction_context','decimal_context','percent_context','representation']:
  nums('MY602',suffix,lambda i:number(f'A jug holds {2+i} litres. Half is poured out. How many litres remain?',f'{(2+i)//2}.{5 if (2+i)%2 else 0}',[text('One half is 50% or 0.5 of the whole.')],'Find half of the starting quantity.',representation=['bar_model','hundredths_grid'][i%2],format='decimal',unit='litres'))
 for suffix in ['equivalent_ratio','missing_part','unit_rate']:
  def ratios(i):
   a=2+i%5;b=2+(i*3)%7
   return number(f'{a}:{b} = {a*3}:?' if suffix!='unit_rate' else f'{a} notebooks cost ${a*b}. What is the price of one notebook?',b*3 if suffix!='unit_rate' else b,[table(['Notebooks','Price'],[[a,a*b]])],'Scale both ratio parts equally.' if suffix!='unit_rate' else 'Divide total price by the number of notebooks.',representation=['ratio_bar_model','table_of_values'][i%2])
  nums('MY604',suffix,ratios)
