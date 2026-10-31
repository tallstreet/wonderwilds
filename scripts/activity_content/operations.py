from .common import *
def build():
 for skill in ['MY104','MY105','MY202','MY302']:
  adding=skill in ['MY104','MY202','MY302']; limit=20 if skill in ['MY104','MY105'] else 100 if skill=='MY202' else 9999
  for suffix in (['count_on','make_ten','addition_story'] if skill=='MY104' else ['count_back','missing_addend','take_away'] if skill=='MY105' else ['add','subtract','missing_value']):
   def make(i,suffix=suffix):
    a=1+i%10 if limit==20 else 20+i*3 if limit==100 else [123,789,456,908,1000,2304,4050,5607,7080,8102,9000,9999][i]
    b=1+(i*3)%8 if limit==20 else 1+i*2 if limit==100 else [111,123,222,108,1,102,305,906,100,801,999,1234][i]
    plus=(skill=='MY104' or suffix in ['add','missing_value'])
    if plus and a+b>limit:a=limit-b
    if not plus and a<b:a,b=b,a
    value=a+b if plus else a-b
    expression=f'{a} {"+" if plus else "−"} {b} = ?'
    if suffix in ['missing_addend','missing_value']:expression=f'{b} + ? = {a if not plus else a+b}';value=a-b if not plus else a
    stim=[text(expression)]
    if limit==20:stim.append(coll([a],layout='ten_frame'))
    return number('Find the missing number.',value,stim,'Work with ones and tens; regroup when needed.',representation=['ten_frame','number_line'][i%2] if limit==20 else ['place_value_chart','number_line'][i%2] if limit==100 else ['column_algorithm','place_value_chart'][i%2])
   nums(skill,suffix,make)
 for skill,factors in [('MY203',[2,3,5,10]),('MY303',list(range(2,11))),('MY402',list(range(2,10))),('MY501',list(range(10,22)))]:
  for suffix in ['multiply','missing_factor','apply_fact'] if skill!='MY402' else ['area_model','standard','partial_product']:
   def make(i):
    a=(2+i%9) if skill in ['MY203','MY303'] else 10+i*7 if skill=='MY402' else 20+i*31
    b=factors[i%len(factors)]
    prompt=f'{a} × {b} = ?';answer=a*b
    if suffix=='missing_factor':prompt=f'{a} × ? = {a*b}';answer=b
    if suffix=='partial_product':prompt=f'What is {a//10*10} × {b}, the tens partial product?';answer=(a//10*10)*b
    return number(prompt,answer,[table(['Groups','In each group'],[[a,b]])],'Multiply the equal groups; use place value for larger numbers.',representation=['array_model','equal_groups'][i%2] if skill=='MY203' else ['array_model','multiplication_table'][i%2] if skill=='MY303' else ['area_model','column_algorithm'][i%2])
   nums(skill,suffix,make)
 nums('MY501','division',lambda i:number(f'{(12+i)*(10+i)} ÷ {10+i} = ?',12+i,[text('Use a multiplication fact to check your quotient.')],'The quotient times the divisor gives the dividend.',representation='column_algorithm'))
 nums('MY501','choose_method',lambda i:choice(f'Which expression helps solve {20+i} × 12?',f'{20+i} × 10 + {20+i} × 2',[f'{20+i} + 12',f'{20+i} × 10 + 2'],hint='Partition one factor and multiply each part.',representation='area_model'))
 for suffix in ['share','group','inverse']:
  nums('MY204',suffix,lambda i:number(f'{(2+i%4)*(1+i%4)} objects are shared into {2+i%4} equal groups. How many in each group?',1+i%4,[coll([(2+i%4)*(1+i%4)])],'Divide into equal groups and check with multiplication.',representation=['equal_groups','array_model'][i%2]))
 for suffix in ['facts','remainder','inverse']:
  def division(i):
   divisor=2+i%9;quotient=2+i%7;remainder=(i%divisor) if suffix=='remainder' else 0;dividend=divisor*quotient+remainder
   fields=[{'id':'quotient','label':'Quotient','format':'integer'},{'id':'remainder','label':'Remainder','format':'integer'}]
   return number(f'{dividend} ÷ {divisor}: what are the quotient and remainder?',[quotient,remainder],[text('Make complete equal groups, then count what remains.')],'The remainder must be smaller than the divisor.',representation=['array_model','number_line'][i%2],fields=fields)
  nums('MY304',suffix,division)
 for skill in ['MY601','MY605']:
  for suffix in (['evaluate','brackets','equivalent_expression'] if skill=='MY601' else ['substitute','solve_equation','match_expression']):
   def algebra(i):
    a=2+i;b=2+i%5;c=2+i%3
    if skill=='MY601':
     if suffix=='brackets':return choice(f'Which expression equals {(a+b)*c}?',f'({a} + {b}) × {c}',[f'{a} + {b} × {c}',f'{a} + {b} + {c}'],hint='Brackets tell us which calculation to do first.',representation='expression_builder')
     if suffix=='equivalent_expression':return choice(f'Which expression has the same value as {a} × ({b} + {c})?',f'{a} × {b} + {a} × {c}',[f'{a} × {b} + {c}',f'{a} + {b} × {c}'],hint='Multiply each term inside the brackets.',representation='worked_example_card')
     return number(f'{a} + {b} × {c} = ?',a+b*c,[text('Use the order of operations.')],'Multiply before adding.',representation='worked_example_card')
    if suffix=='substitute':return number(f'If x = {a}, what is {b}x + {c}?',b*a+c,[text(f'x = {a}')],'Replace x with its value, then multiply and add.',representation='algebra_tiles')
    if suffix=='solve_equation':return number(f'{b}x + {c} = {b*a+c}. What is x?',a,hint='Undo addition, then undo multiplication on both sides.',representation='equation_balance')
    return choice(f'A basket has x berries. Add {a} more. Which expression gives the total?',f'x + {a}',[f'x × {a}',f'x − {a}'],hint='More berries means addition.',representation='algebra_tiles')
   nums(skill,suffix,algebra)
 for suffix in ['prime_composite','divisibility','factorisation']:
  def prime(i):
   n=2+i
   divisors=[d for d in range(2,n) if n%d==0]
   if suffix=='prime_composite':return choice(f'Is {n} prime or composite?','prime' if not divisors else 'composite',['composite' if not divisors else 'prime'],hint='A prime number has exactly two positive factors.',representation='sieve_of_eratosthenes')
   if suffix=='divisibility':return multi(f'Which of these are factors of {n}?',['1','2',str(n)],[0,2]+([1] if n%2==0 and n!=2 else []),hint='A factor divides the number with no remainder.',representation='factor_tree') if n!=2 else choice('How many positive factors does 2 have?',2,[1,3],representation='factor_tree')
   d=2+i%5;e=2+(i*3)%7
   return choice(f'Which factor pair makes {d*e}?',f'{d} × {e}',[f'{d} + {e}',f'{d+1} × {e+1}'],hint='Multiply each pair to check.',representation='factor_tree')
  nums('MY603',suffix,prime,30 if suffix=='prime_composite' else 12)
