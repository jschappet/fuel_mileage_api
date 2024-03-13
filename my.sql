select miles_driven, gallons, total_cost ,
(miles_driven/gallons) as MPG, 
(total_cost / miles_driven )  as cost_per_mile
 from fuel_logs;

select * from cars;

