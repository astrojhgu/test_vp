extern crate vpsearch;
extern crate num_traits;


use std::cmp::Ordering;
use num_traits::float::{Float, FloatConst};
use num_traits::Bounded;
use std::collections::BinaryHeap;


use vpsearch::BestCandidate;


//spherical coordinates with unit of radians
//polar angle=0 on z+ axis, =
//azimuth angle=0 on x+ axis, and =pi/2 on y+ axis
#[derive(Copy, Clone, Debug)]
struct SphCoord{
    pol:f64,//polar angle
    az:f64//azimuth angle
}

impl SphCoord{
    fn new(pol:f64, az:f64)->SphCoord{//constructor
        SphCoord{pol:pol, az:az}
    }
}



impl vpsearch::MetricSpace for SphCoord{
    type UserData=();
    type Distance=f64;

    fn distance(&self, other:&Self, _:&())->f64{
        /*
        distance is defined tne angle between two points, i.e., the arc length connecting two points on a unit sphere
        */
        //convert to unit vector
        let x1=self.pol.sin()*self.az.cos();
        let y1=self.pol.sin()*self.az.sin();
        let z1=self.pol.cos();

        let x2=other.pol.sin()*other.az.cos();
        let y2=other.pol.sin()*other.az.sin();
        let z2=other.pol.cos();


        //calculate the inner product and arccos to calculate the angle
        (x1*x2+y1*y2+z1*z2).acos()
    }
}


//used to store the index and distance
//in the implement of BestCandidate
#[derive(Debug)]
struct IdxDistPair<T>
where T:Float+Copy {
    idx:usize,
    dist:T
}

// inorder to store IdxDistPair in a BinaryTree, following
//Eq and Ord must be implemented
impl<T:Float+Copy> PartialEq for IdxDistPair<T>{
    fn eq(&self, other:&Self)->bool{
        self.dist==other.dist
    }
}

impl<T:Float+Copy> PartialOrd for IdxDistPair<T>{
    fn partial_cmp(&self, other:&Self) -> Option<Ordering>{
        self.dist.partial_cmp(&other.dist)
    }
}

impl<T:Float+Copy> Eq for IdxDistPair<T>{}

impl<T:Float+Copy> Ord for IdxDistPair<T>{
    fn cmp(&self, other:&Self) -> Ordering{
        self.partial_cmp(other).unwrap()
    }
}

//My customized BestCandidate
struct ReturnNNearestIdx<T:Float+Copy> {
    n:usize,//how many points we want to search
    candidates:BinaryHeap<IdxDistPair<T>>,//candidate points
}

impl<T:Float+Copy> ReturnNNearestIdx<T>{

    fn new(n:usize)->ReturnNNearestIdx<T>{
        ReturnNNearestIdx{n:n, candidates:BinaryHeap::<IdxDistPair<T>>::new()}
    }
}


impl BestCandidate<SphCoord, ()> for ReturnNNearestIdx<f64>{
    type Output=BinaryHeap<IdxDistPair<f64>>;

    fn consider(&mut self, _:&SphCoord, distance: f64, candidate_idx:usize, _: &()){
        //if current point is to be added to the candidate list
        enum CompareResult{
            ACCEPTED,
            REJECTED
        };

        let comp_res=
        if self.candidates.len()<self.n{
            //if we have not yet collected enough number of points
            CompareResult::ACCEPTED
        }
        else{
            //if we have already collected enough number of points, but we need
            //to check if new point is better
            match self.candidates.peek() {
                //BinaryTree::peek can always return the greatest element
                //We have already implemented the Ord for IdxDistPair,
                //the ordering is computed according only to the dist member of IdxDistPair
                //i.e., the distance
                Some(x) => {
                    if distance < x.dist {
                        //if current point is closer than the farthest point in the candidate list
                        //then accept current point and drop the farthest point
                        CompareResult::ACCEPTED
                    } else {
                        //otherwise reject it
                        CompareResult::REJECTED
                    }
                },
                _ => {
                    //this branch should never be reached
                    unreachable!("should never reach here");
                    //CompareResult::ACCEPTED
                }
            }
        };



        match comp_res{
            CompareResult::ACCEPTED => {
                //if accepted
                if self.candidates.len()>=self.n {
                    //if enough points have been collected
                    //drop the farthest one in the candidate list
                    self.candidates.pop();
                }
                //push current point
                self.candidates.push(IdxDistPair{idx:candidate_idx, dist:distance});
            },
            _ => ()//otherwise, do nothing
        }
    }

    fn distance(&self)-> f64{
        match self.candidates.peek(){
            Some(x)=>x.dist,//return the distance of the farthest point in the candidate list
            _ => <f64 as Bounded>::max_value()//if no point has been collected, return a very big value
        }
    }

    fn result(self, _ : &()) -> BinaryHeap<IdxDistPair<f64>>{
        self.candidates
    }
}


fn main() {
    //compose the tree
    //first compose a vector of points
    let points=vec![
                    SphCoord::new(90_f64.to_radians(), 0_f64.to_radians()),
                    SphCoord::new(90_f64.to_radians(), 180_f64.to_radians()),
                    SphCoord::new(90_f64.to_radians(), 90_f64.to_radians()),
                    SphCoord::new(90_f64.to_radians(), -90_f64.to_radians()),//note this
                    SphCoord::new(0_f64.to_radians(), 0_f64.to_radians()),
                    SphCoord::new(180_f64.to_radians(), 0_f64.to_radians()),
    ];

    /*
    //If you uncommon following codes, the results will be right
    //the difference is at the 4th point: az=270 vs az=90
    //mathematically they identical
    //but will cause different result!
    let points=vec![
        SphCoord::new(90_f64.to_radians(), 0_f64.to_radians()),
        SphCoord::new(90_f64.to_radians(), 180_f64.to_radians()),
        SphCoord::new(90_f64.to_radians(), 90_f64.to_radians()),
        SphCoord::new(90_f64.to_radians(), 270_f64.to_radians()),//note this
        SphCoord::new(0_f64.to_radians(), 0_f64.to_radians()),
        SphCoord::new(180_f64.to_radians(), 0_f64.to_radians()),
    ];
    */

    //then compose the tree
    let vp=vpsearch::Tree::new(&points);


    //following certain value will cause the program collect not enough points

    let az_min = -f64::PI();
    let az_max = f64::PI();

    let naz = 100;

    //for j in 0..naz
    let j=54;

    let pol = 90_f64.to_radians();
    let az = (az_max - az_min) / (naz as f64) * (j as f64) + az_min;

    println!("following points are used to compose the vp tree:");
    for p in &points{
        println!("{:?}", p);
    }

    //If I query 3 nearest points to following point, only 2 will be returned
    let sph = SphCoord::new(pol, az);

    println!("we need to find 3 nearest points of following point:");
    println!("{:?}", sph);
    let mut result=vp.find_nearest_custom(&sph, &(), ReturnNNearestIdx::new(3));
    println!("Totally {} points found", result.len());//this will output 2
    println!("They are");
    while let Some(x)=result.pop(){
        println!("idx and dist={:?} point={:?}", x, points[x.idx]);
    }
}
