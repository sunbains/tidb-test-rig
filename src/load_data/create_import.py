#!/usr/bin/env python3
"""
Test data generator for TiDB import tests.

This program generates CSV/TSV test data files for testing TiDB IMPORT functionality.
It creates realistic test data with various data types and can generate large datasets
for performance testing.

Usage:
    python create_import.py [--rows 100000] [--format csv|tsv] [--output filename]
    python create_import.py --help
"""

import argparse
import random
import string
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path


class TestDataGenerator:
    """Generate test data for import tests."""
    
    def __init__(self, seed=None):
        """Initialize the generator with optional seed for reproducibility."""
        if seed is not None:
            random.seed(seed)
        else:
            random.seed(int(time.time()))
        
        # Sample data for realistic test data
        self.first_names = [
            "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry",
            "Ivy", "Jack", "Kate", "Liam", "Mia", "Noah", "Olivia", "Paul",
            "Quinn", "Ruby", "Sam", "Tara", "Uma", "Victor", "Wendy", "Xavier",
            "Yara", "Zoe", "Adam", "Beth", "Carl", "Dora", "Eric", "Fiona",
            "George", "Helen", "Ian", "Jane", "Kevin", "Lisa", "Mark", "Nina"
        ]
        
        self.last_names = [
            "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller",
            "Davis", "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez",
            "Wilson", "Anderson", "Thomas", "Taylor", "Moore", "Jackson", "Martin",
            "Lee", "Perez", "Thompson", "White", "Harris", "Sanchez", "Clark",
            "Ramirez", "Lewis", "Robinson", "Walker", "Young", "Allen", "King",
            "Wright", "Scott", "Torres", "Nguyen", "Hill", "Flores", "Green"
        ]
        
        self.cities = [
            "New York", "Los Angeles", "Chicago", "Houston", "Phoenix", "Philadelphia",
            "San Antonio", "San Diego", "Dallas", "San Jose", "Austin", "Jacksonville",
            "Fort Worth", "Columbus", "Charlotte", "San Francisco", "Indianapolis",
            "Seattle", "Denver", "Washington", "Boston", "El Paso", "Nashville",
            "Detroit", "Oklahoma City", "Portland", "Las Vegas", "Memphis",
            "Louisville", "Baltimore", "Milwaukee", "Albuquerque", "Tucson",
            "Fresno", "Sacramento", "Mesa", "Kansas City", "Atlanta", "Long Beach",
            "Colorado Springs", "Raleigh"
        ]
        
        self.departments = [
            "Engineering", "Sales", "Marketing", "Finance", "Human Resources",
            "Operations", "Customer Support", "Product Management", "Design",
            "Legal", "Research", "Quality Assurance", "Business Development",
            "Information Technology", "Administration"
        ]
        
        self.job_titles = [
            "Software Engineer", "Sales Representative", "Marketing Manager",
            "Financial Analyst", "HR Specialist", "Operations Manager",
            "Customer Success Manager", "Product Manager", "UX Designer",
            "Legal Counsel", "Research Scientist", "QA Engineer",
            "Business Development Manager", "IT Administrator", "Executive Assistant"
        ]

    def generate_id(self, start_id=1):
        """Generate sequential IDs."""
        return start_id

    def generate_name(self):
        """Generate a random full name."""
        return f"{random.choice(self.first_names)} {random.choice(self.last_names)}"

    def generate_email(self, name):
        """Generate email from name."""
        # Clean name for email
        clean_name = name.lower().replace(" ", ".")
        domains = ["example.com", "test.org", "demo.net", "sample.co", "mock.io"]
        return f"{clean_name}@{random.choice(domains)}"

    def generate_phone(self):
        """Generate a random phone number."""
        area_code = random.randint(200, 999)
        prefix = random.randint(200, 999)
        line = random.randint(1000, 9999)
        return f"({area_code}) {prefix}-{line}"

    def generate_date(self, start_year=2020, end_year=2024):
        """Generate a random date."""
        start_date = datetime(start_year, 1, 1)
        end_date = datetime(end_year, 12, 31)
        time_between = end_date - start_date
        days_between = time_between.days
        random_days = random.randrange(days_between)
        random_date = start_date + timedelta(days=random_days)
        return random_date.strftime("%Y-%m-%d")

    def generate_salary(self):
        """Generate a random salary."""
        return random.randint(30000, 150000)

    def generate_decimal(self, min_val=0.0, max_val=1000.0, precision=2):
        """Generate a random decimal number."""
        return round(random.uniform(min_val, max_val), precision)

    def generate_text(self, min_length=10, max_length=200):
        """Generate random text."""
        words = [
            "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing",
            "elit", "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore",
            "et", "dolore", "magna", "aliqua", "ut", "enim", "ad", "minim", "veniam",
            "quis", "nostrud", "exercitation", "ullamco", "laboris", "nisi", "ut",
            "aliquip", "ex", "ea", "commodo", "consequat", "duis", "aute", "irure",
            "dolor", "in", "reprehenderit", "in", "voluptate", "velit", "esse",
            "cillum", "dolore", "eu", "fugiat", "nulla", "pariatur"
        ]
        length = random.randint(min_length, max_length)
        return " ".join(random.choices(words, k=length))

    def generate_boolean(self):
        """Generate a random boolean value."""
        return random.choice([True, False])

    def generate_csv_row(self, row_id):
        """Generate a CSV row with various data types."""
        name = self.generate_name()
        email = self.generate_email(name)
        
        row = [
            str(row_id),  # id
            name,  # name
            email,  # email
            self.generate_phone(),  # phone
            random.choice(self.cities),  # city
            random.choice(self.departments),  # department
            random.choice(self.job_titles),  # job_title
            str(self.generate_salary()),  # salary
            str(self.generate_decimal(0, 100)),  # performance_score
            self.generate_date(),  # hire_date
            str(self.generate_boolean()),  # is_active
            f'"{self.generate_text(20, 100)}"',  # notes (quoted for CSV)
            str(random.randint(1, 50)),  # years_experience
            str(random.randint(0, 10))  # projects_completed
        ]
        return ",".join(row)

    def generate_tsv_row(self, row_id):
        """Generate a TSV row with various data types."""
        name = self.generate_name()
        email = self.generate_email(name)
        
        row = [
            str(row_id),  # id
            name,  # name
            email,  # email
            self.generate_phone(),  # phone
            random.choice(self.cities),  # city
            random.choice(self.departments),  # department
            random.choice(self.job_titles),  # job_title
            str(self.generate_salary()),  # salary
            str(self.generate_decimal(0, 100)),  # performance_score
            self.generate_date(),  # hire_date
            str(self.generate_boolean()),  # is_active
            self.generate_text(20, 100),  # notes
            str(random.randint(1, 50)),  # years_experience
            str(random.randint(0, 10))  # projects_completed
        ]
        return "\t".join(row)

    def generate_simple_csv_row(self, row_id):
        """Generate a simple CSV row for basic tests."""
        name = self.generate_name()
        age = random.randint(18, 65)
        return f"{row_id},{name},{age}"

    def generate_simple_tsv_row(self, row_id):
        """Generate a simple TSV row for basic tests."""
        name = self.generate_name()
        age = random.randint(18, 65)
        return f"{row_id}\t{name}\t{age}"

    def create_test_file(self, num_rows, output_file, file_format="csv", simple=False):
        """Create a test data file with the specified number of rows."""
        print(f"Generating {num_rows:,} rows of {file_format.upper()} data...")
        print(f"Output file: {output_file}")
        
        start_time = time.time()
        
        with open(output_file, 'w', encoding='utf-8') as f:
            # Write header for complex format
            if not simple:
                if file_format.lower() == "csv":
                    header = "id,name,email,phone,city,department,job_title,salary,performance_score,hire_date,is_active,notes,years_experience,projects_completed"
                else:  # tsv
                    header = "id\tname\temail\tphone\tcity\tdepartment\tjob_title\tsalary\tperformance_score\thire_date\tis_active\tnotes\tyears_experience\tprojects_completed"
                f.write(header + "\n")
            
            # Generate rows
            for i in range(1, num_rows + 1):
                if simple:
                    if file_format.lower() == "csv":
                        row = self.generate_simple_csv_row(i)
                    else:  # tsv
                        row = self.generate_simple_tsv_row(i)
                else:
                    if file_format.lower() == "csv":
                        row = self.generate_csv_row(i)
                    else:  # tsv
                        row = self.generate_tsv_row(i)
                
                f.write(row + "\n")
                
                # Progress indicator for large files
                if i % 10000 == 0:
                    elapsed = time.time() - start_time
                    rate = i / elapsed if elapsed > 0 else 0
                    print(f"Generated {i:,} rows ({rate:.0f} rows/sec)")
        
        elapsed_time = time.time() - start_time
        file_size = Path(output_file).stat().st_size
        
        print(f"\n✅ Generation complete!")
        print(f"📊 Statistics:")
        print(f"   Rows generated: {num_rows:,}")
        print(f"   File size: {file_size:,} bytes ({file_size / 1024 / 1024:.2f} MB)")
        print(f"   Time taken: {elapsed_time:.2f} seconds")
        print(f"   Generation rate: {num_rows / elapsed_time:.0f} rows/sec")
        print(f"   Output file: {output_file}")


def main():
    """Main function to handle command line arguments and generate test data."""
    parser = argparse.ArgumentParser(
        description="Generate test data for TiDB import tests",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python create_import.py --rows 1000 --format csv --output test_data.csv
  python create_import.py --rows 50000 --format tsv --output large_dataset.tsv
  python create_import.py --rows 100000 --simple --output simple_data.csv
  python create_import.py --rows 1000000 --output huge_dataset.csv
        """
    )
    
    parser.add_argument(
        "--rows", "-r",
        type=int,
        default=100000,
        help="Number of rows to generate (default: 100000)"
    )
    
    parser.add_argument(
        "--format", "-f",
        choices=["csv", "tsv"],
        default="csv",
        help="Output format: csv or tsv (default: csv)"
    )
    
    parser.add_argument(
        "--output", "-o",
        type=str,
        help="Output filename (default: test_data_<rows>.<format>)"
    )
    
    parser.add_argument(
        "--simple", "-s",
        action="store_true",
        help="Generate simple format (id,name,age) instead of complex format"
    )
    
    parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducible data generation"
    )
    
    args = parser.parse_args()
    
    # Validate arguments
    if args.rows <= 0:
        print("❌ Error: Number of rows must be positive")
        sys.exit(1)
    
    if args.rows > 10000000:
        print("⚠️  Warning: Generating more than 10 million rows may take a long time")
        response = input("Continue? (y/N): ")
        if response.lower() != 'y':
            print("Generation cancelled")
            sys.exit(0)
    
    # Generate output filename if not provided
    if not args.output:
        args.output = f"test_data_{args.rows:,}.{args.format}"
    
    # Create generator and generate data
    try:
        generator = TestDataGenerator(seed=args.seed)
        generator.create_test_file(
            num_rows=args.rows,
            output_file=args.output,
            file_format=args.format,
            simple=args.simple
        )
    except KeyboardInterrupt:
        print("\n❌ Generation interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error generating test data: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main() 