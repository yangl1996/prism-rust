import sys
import json
import itertools
import urllib.request
def compare_list(x,y):
    return [(idx,i,j) for idx, (i, j) in enumerate(itertools.zip_longest(x, y)) if i != j]

def compare_proposer_levels(x,y):
    print('Proposers diff')
    for idx, l, r in compare_list(x['proposer_levels'],y['proposer_levels']):
        if l is None:
            l = []
        if r is None:
            r = []
        l_=[x[-6:] for x in set(l).difference(set(r))]
        r_=[x[-6:] for x in set(r).difference(set(l))]
        if len(l_) + len(r_) > 0:
            print('\tLevel {}, left exclusive: {}, right exclusive: {}'.format(idx,l_,r_))

def compare_voter_nodes(x,y):
    print('Voters diff')
    x_set=set(x['voter_nodes'].keys())
    y_set=set(y['voter_nodes'].keys())
    l=x_set.difference(y_set)
    r=y_set.difference(x_set)
    l_chains = [None] * len(x['voter_longest'])
    r_chains = [None] * len(y['voter_longest'])
    for h in l:
        chain = x['voter_nodes'][h]['chain']
        n = (x['voter_nodes'][h]['level'], h)
        if l_chains[chain] is None:
            l_chains[chain] = [n]
        else:
            l_chains[chain].append(n)
    for h in r:
        chain = y['voter_nodes'][h]['chain']
        n = (y['voter_nodes'][h]['level'], h)
        if r_chains[chain] is None:
            r_chains[chain] = [n]
        else:
            r_chains[chain].append(n)
    for i in range(len(l_chains)):
        if l_chains[i] is None:
            l_chains[i]=[]
        else:
            l_chains[i]=sorted(l_chains[i])
    for i in range(len(r_chains)):
        if r_chains[i] is None:
            r_chains[i]=[]
        else:
            r_chains[i]=sorted(r_chains[i])
    for i in range(len(l_chains)):
        if len(l_chains[i]) + len(r_chains[i]) > 0:
            print('\tChain {:3}'.format(i))
        for l_voter in l_chains[i]:
            print('\t\tLeft  level {:3}, hash {}, status {}'.format(l_voter[0],l_voter[1][-6:],x['voter_nodes'][l_voter[1]]['status']))
        for r_voter in r_chains[i]:
            print('\t\tRight level {:3}, hash {}, status {}'.format(r_voter[0],r_voter[1][-6:],y['voter_nodes'][r_voter[1]]['status']))

def compare_voter_longest(x,y):
    print('Voter best diff')
    for idx, l, r in compare_list(x['voter_longest'],y['voter_longest']):
        print('\tChain {:3}, left: {} (lvl{:3}), right: {} (lvl{:3})'.format(idx,l[-6:], x['voter_nodes'][l]['level'],r[-6:],y['voter_nodes'][r]['level']))

def compare_proposer_leaders(x,y):
    print('Leader diff')
    x_list = [(int(i[0]), i[1]) for i in x['proposer_leaders'].items()]
    y_list = [(int(i[0]), i[1]) for i in y['proposer_leaders'].items()]
    for idx, l, r in compare_list(x_list, y_list):
        if l is not None:
            l = 'hash {} (lvl {:3})'.format(l[1][-6:], l[0])
        if r is not None:
            r = 'hash {} (lvl {:3})'.format(r[1][-6:], r[0])
        print('\t{:3}, left: {}, right: {}'.format(idx,l,r))

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print('Pass 2 urls as arguments')
        exit(0)
    with urllib.request.urlopen('http://{}'.format(sys.argv[1])) as response_1, urllib.request.urlopen('http://{}'.format(sys.argv[2])) as response_2:
        dump_1 = response_1.read()
        dump_1 = json.loads(dump_1)
        dump_2 = response_2.read()
        dump_2 = json.loads(dump_2)
        compare_proposer_levels(dump_1, dump_2)
        compare_proposer_leaders(dump_1, dump_2)
        compare_voter_nodes(dump_1, dump_2)
        compare_voter_longest(dump_1, dump_2)
